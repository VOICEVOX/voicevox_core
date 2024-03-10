use std::{
    any,
    collections::{BTreeMap, HashMap},
    convert::Infallible,
    fmt::Display,
    marker::PhantomData,
    sync::Arc,
};

use anyhow::bail;
use educe::Educe;
use enum_map::{Enum as _, EnumMap};
use indexmap::IndexMap;
use itertools::{iproduct, Itertools as _};

use crate::{
    error::{ErrorRepr, LoadModelError, LoadModelErrorKind, LoadModelResult},
    infer::{
        ConvertInferenceDomainAssociationTarget, InferenceDomainAssociation, InferenceDomainGroup,
        InferenceOperation, ParamInfo,
    },
    manifest::ModelInnerId,
    metas::{self, SpeakerMeta, StyleId, StyleMeta, VoiceModelMeta},
    voice_model::{ModelData, ModelDataByInferenceDomain, VoiceModelHeader, VoiceModelId},
    Result, StyleType,
};

use super::{
    model_file, InferenceDomain, InferenceDomainAssociationTargetPredicate,
    InferenceDomainMap as _, InferenceInputSignature, InferenceRuntime, InferenceSessionOptions,
    InferenceSignature,
};

pub(crate) struct Status<R: InferenceRuntime, G: InferenceDomainGroup> {
    loaded_models: std::sync::Mutex<LoadedModels<R, G>>,
    session_options: G::Map<SessionOptionsByDomain>,
}

impl<R: InferenceRuntime, G: InferenceDomainGroup> Status<R, G> {
    pub(crate) fn new(session_options: G::Map<SessionOptionsByDomain>) -> Self {
        Self {
            loaded_models: Default::default(),
            session_options,
        }
    }

    pub(crate) fn insert_model(
        &self,
        model_header: &VoiceModelHeader,
        model_bytes: &G::Map<Option<ModelDataByInferenceDomain>>,
    ) -> Result<()> {
        self.loaded_models
            .lock()
            .unwrap()
            .ensure_acceptable(model_header, model_bytes)?;

        let session_set = model_bytes
            .try_ref_map(CreateSessionSet {
                session_options: &self.session_options,
                marker: PhantomData,
            })
            .map_err(|source| LoadModelError {
                path: model_header.path.clone(),
                context: LoadModelErrorKind::InvalidModelData,
                source: Some(source),
            })?;

        self.loaded_models
            .lock()
            .unwrap()
            .insert(model_header, session_set)?;
        return Ok(());

        struct CreateSessionSet<'a, R, G: InferenceDomainGroup> {
            session_options: &'a G::Map<SessionOptionsByDomain>,
            marker: PhantomData<fn() -> R>,
        }

        impl<R: InferenceRuntime, G: InferenceDomainGroup>
            ConvertInferenceDomainAssociationTarget<
                G,
                Option<ModelDataByInferenceDomain>,
                Option<(ModelInnerIdsByDomain, SessionSetByDomain<R>)>,
                anyhow::Error,
            > for CreateSessionSet<'_, R, G>
        {
            fn try_ref_map<D: InferenceDomain<Group = G>>(
                &self,
                model_data: &<Option<ModelDataByInferenceDomain> as InferenceDomainAssociation>::Target<D>,
            ) -> anyhow::Result<
                <Option<(ModelInnerIdsByDomain, SessionSetByDomain<R>)> as InferenceDomainAssociation>::Target<D>,
            >{
                model_data
                    .as_ref()
                    .map(
                        |ModelData {
                             model_inner_ids,
                             model_bytes,
                         }| {
                            let session_set =
                                SessionSet::new(model_bytes, D::visit(self.session_options))?;
                            Ok((model_inner_ids.clone(), session_set))
                        },
                    )
                    .transpose()
            }
        }
    }

    pub(crate) fn unload_model(&self, voice_model_id: &VoiceModelId) -> Result<()> {
        self.loaded_models.lock().unwrap().remove(voice_model_id)
    }

    pub(crate) fn metas(&self) -> VoiceModelMeta {
        self.loaded_models.lock().unwrap().metas()
    }

    /// あるスタイルに対応する`VoiceModelId`と`ModelInnerId`の組を返す。
    ///
    /// `StyleId` → `ModelInnerId`のマッピングが存在しない場合は、`ModelInnerId`としては
    /// `style_id`と同じ値を返す。
    pub(crate) fn ids_for<D: InferenceDomain<Group = G>>(
        &self,
        style_id: StyleId,
    ) -> Result<(VoiceModelId, ModelInnerId)> {
        self.loaded_models.lock().unwrap().ids_for::<D>(style_id)
    }

    pub(crate) fn is_loaded_model(&self, voice_model_id: &VoiceModelId) -> bool {
        self.loaded_models
            .lock()
            .unwrap()
            .contains_voice_model(voice_model_id)
    }

    // FIXME: この関数はcompatible_engineとテストでのみ使われるが、テストのために`StyleType`を
    // 引数に含めるようにする
    pub(crate) fn is_loaded_model_by_style_id(&self, style_id: StyleId) -> bool {
        self.loaded_models.lock().unwrap().contains_style(style_id)
    }

    /// 推論を実行する。
    ///
    /// # Performance
    ///
    /// CPU/GPU-boundな操作であるため、非同期ランタイム上では直接実行されるべきではない。
    ///
    /// # Panics
    ///
    /// `self`が`model_id`を含んでいないとき、パニックする。
    pub(crate) fn run_session<I>(
        &self,
        model_id: &VoiceModelId,
        input: I,
    ) -> Result<<I::Signature as InferenceSignature>::Output>
    where
        I: InferenceInputSignature,
        I::Signature: InferenceSignature,
        <I::Signature as InferenceSignature>::Domain: InferenceDomain<Group = G>,
    {
        let sess = self.loaded_models.lock().unwrap().get(model_id);
        sess.run(input)
    }
}

/// 読み込んだモデルの`Session`とそのメタ情報を保有し、追加/削除/取得の操作を提供する。
///
/// この構造体のメソッドは、すべて一瞬で完了すべきである。
#[derive(Educe)]
#[educe(Default(bound = "R: InferenceRuntime, G: InferenceDomainGroup"))]
struct LoadedModels<R: InferenceRuntime, G: InferenceDomainGroup>(
    IndexMap<VoiceModelId, LoadedModel<R, G>>,
);

struct LoadedModel<R: InferenceRuntime, G: InferenceDomainGroup> {
    metas: VoiceModelMeta,
    by_domain: G::Map<Option<(ModelInnerIdsByDomain, SessionSetByDomain<R>)>>,
}

impl<R: InferenceRuntime, G: InferenceDomainGroup> LoadedModels<R, G> {
    fn metas(&self) -> VoiceModelMeta {
        metas::merge(self.0.values().flat_map(|LoadedModel { metas, .. }| metas))
    }

    fn ids_for<D: InferenceDomain<Group = G>>(
        &self,
        style_id: StyleId,
    ) -> Result<(VoiceModelId, ModelInnerId)> {
        let (model_id, LoadedModel { by_domain, .. }) = self
            .0
            .iter()
            .find(|(_, LoadedModel { metas, .. })| {
                metas.iter().flat_map(SpeakerMeta::styles).any(|style| {
                    *style.id() == style_id && D::style_types().contains(style.r#type())
                })
            })
            .ok_or(ErrorRepr::StyleNotFound {
                style_id,
                style_types: D::style_types(),
            })?;

        let model_inner_id = D::visit(by_domain)
            .as_ref()
            .and_then(|(model_inner_ids, _)| model_inner_ids.get(&style_id).copied())
            .unwrap_or_else(|| ModelInnerId::new(style_id.raw_id()));

        Ok((model_id.clone(), model_inner_id))
    }

    /// # Panics
    ///
    /// 次の場合にパニックする。
    ///
    /// - `self`が`model_id`を含んでいないとき
    /// - 対応する`InferenceDomain`が欠けているとき
    fn get<I>(&self, model_id: &VoiceModelId) -> SessionCell<R, I>
    where
        I: InferenceInputSignature,
        <I::Signature as InferenceSignature>::Domain: InferenceDomain<Group = G>,
    {
        let (_, session_set) =
            <I::Signature as InferenceSignature>::Domain::visit(&self.0[model_id].by_domain)
                .as_ref()
                .unwrap_or_else(|| {
                    let type_name =
                        any::type_name::<<I::Signature as InferenceSignature>::Domain>()
                            .split("::")
                            .last()
                            .unwrap();
                    panic!(
                        "missing session set for `{type_name}` (should be checked in \
                         `ensure_acceptable`)",
                    );
                });
        session_set.get()
    }

    fn contains_voice_model(&self, model_id: &VoiceModelId) -> bool {
        self.0.contains_key(model_id)
    }

    fn contains_style(&self, style_id: StyleId) -> bool {
        self.styles().any(|style| *style.id() == style_id)
    }

    /// 音声モデルを受け入れ可能かをチェックする。
    ///
    /// # Errors
    ///
    /// 次の場合にエラーを返す。
    ///
    /// - 現在持っている音声モデルIDかスタイルIDが`model_header`と重複するとき
    /// - 必要であるはずの`InferenceDomain`のモデルデータが欠けているとき
    fn ensure_acceptable(
        &self,
        model_header: &VoiceModelHeader,
        model_bytes_or_sessions: &G::Map<Option<impl InferenceDomainAssociation>>,
    ) -> LoadModelResult<()> {
        let error = |context| LoadModelError {
            path: model_header.path.clone(),
            context,
            source: None,
        };

        if self.0.contains_key(&model_header.id) {
            return Err(error(LoadModelErrorKind::ModelAlreadyLoaded {
                id: model_header.id.clone(),
            }));
        }

        let loaded = self.speakers();
        let external = model_header.metas.iter();
        for (loaded, external) in iproduct!(loaded, external) {
            if loaded.speaker_uuid() == external.speaker_uuid() {
                loaded.warn_diff_except_styles(external);
            }
        }

        let loaded = self.styles();
        let external = model_header
            .metas
            .iter()
            .flat_map(|speaker| speaker.styles());
        if let Some(style_type) =
            external
                .clone()
                .map(StyleMeta::r#type)
                .copied()
                .find(|&style_type| {
                    !model_bytes_or_sessions.any(ContainsForStyleType {
                        style_type,
                        marker: PhantomData,
                    })
                })
        {
            return Err(error(LoadModelErrorKind::MissingModelData { style_type }));
        }
        if let Some((style, _)) =
            iproduct!(loaded, external).find(|(loaded, external)| loaded.id() == external.id())
        {
            return Err(error(LoadModelErrorKind::StyleAlreadyLoaded {
                id: *style.id(),
            }));
        }
        return Ok(());

        struct ContainsForStyleType<A> {
            style_type: StyleType,
            marker: PhantomData<fn() -> A>,
        }

        impl<A: InferenceDomainAssociation> InferenceDomainAssociationTargetPredicate
            for ContainsForStyleType<A>
        {
            type Association = Option<A>;

            fn test<D: InferenceDomain>(
                &self,
                x: &<Self::Association as InferenceDomainAssociation>::Target<D>,
            ) -> bool {
                D::style_types().contains(&self.style_type) && x.is_some()
            }
        }
    }

    fn insert(
        &mut self,
        model_header: &VoiceModelHeader,
        session_sets: G::Map<Option<(ModelInnerIdsByDomain, SessionSetByDomain<R>)>>,
    ) -> Result<()> {
        self.ensure_acceptable(model_header, &session_sets)?;

        let prev = self.0.insert(
            model_header.id.clone(),
            LoadedModel {
                metas: model_header.metas.clone(),
                by_domain: session_sets,
            },
        );
        assert!(prev.is_none());
        Ok(())
    }

    fn remove(&mut self, model_id: &VoiceModelId) -> Result<()> {
        if self.0.remove(model_id).is_none() {
            return Err(ErrorRepr::ModelNotFound {
                model_id: model_id.clone(),
            }
            .into());
        }
        Ok(())
    }

    fn speakers(&self) -> impl Iterator<Item = &SpeakerMeta> + Clone {
        self.0.values().flat_map(|LoadedModel { metas, .. }| metas)
    }

    fn styles(&self) -> impl Iterator<Item = &StyleMeta> {
        self.speakers().flat_map(|speaker| speaker.styles())
    }
}

struct SessionSet<R: InferenceRuntime, D: InferenceDomain>(
    EnumMap<D::Operation, Arc<std::sync::Mutex<R::Session>>>,
);

impl<R: InferenceRuntime, D: InferenceDomain> SessionSet<R, D> {
    fn new(
        model_bytes: &EnumMap<D::Operation, Vec<u8>>,
        options: &EnumMap<D::Operation, InferenceSessionOptions>,
    ) -> anyhow::Result<Self> {
        let mut sessions = model_bytes
            .iter()
            .map(|(op, model_bytes)| {
                let (expected_input_param_infos, expected_output_param_infos) =
                    <D::Operation as InferenceOperation>::PARAM_INFOS[op];

                let (sess, actual_input_param_infos, actual_output_param_infos) =
                    R::new_session(|| model_file::decrypt(model_bytes), options[op])?;

                check_param_infos(expected_input_param_infos, &actual_input_param_infos)?;
                check_param_infos(expected_output_param_infos, &actual_output_param_infos)?;

                Ok((op.into_usize(), std::sync::Mutex::new(sess).into()))
            })
            .collect::<anyhow::Result<HashMap<_, _>>>()?;

        return Ok(Self(EnumMap::<D::Operation, _>::from_fn(|k| {
            sessions.remove(&k.into_usize()).expect("should exist")
        })));

        fn check_param_infos<D: PartialEq + Display>(
            expected: &[ParamInfo<D>],
            actual: &[ParamInfo<D>],
        ) -> anyhow::Result<()> {
            if !(expected.len() == actual.len()
                && itertools::zip_eq(expected, actual)
                    .all(|(expected, actual)| expected.accepts(actual)))
            {
                let expected = display_param_infos(expected);
                let actual = display_param_infos(actual);
                bail!("expected {{{expected}}}, got {{{actual}}}")
            }
            Ok(())
        }

        fn display_param_infos(infos: &[ParamInfo<impl Display>]) -> impl Display {
            infos
                .iter()
                .map(|ParamInfo { name, dt, ndim }| {
                    let brackets = match *ndim {
                        Some(ndim) => "[]".repeat(ndim),
                        None => "[]...".to_owned(),
                    };
                    format!("{name}: {dt}{brackets}")
                })
                .join(", ")
        }
    }
}

impl<R: InferenceRuntime, D: InferenceDomain> SessionSet<R, D> {
    fn get<I>(&self) -> SessionCell<R, I>
    where
        I: InferenceInputSignature,
        I::Signature: InferenceSignature<Domain = D>,
    {
        SessionCell {
            inner: self.0[I::Signature::OPERATION].clone(),
            marker: PhantomData,
        }
    }
}

struct SessionCell<R: InferenceRuntime, I> {
    inner: Arc<std::sync::Mutex<R::Session>>,
    marker: PhantomData<fn(I)>,
}

impl<R: InferenceRuntime, I: InferenceInputSignature> SessionCell<R, I> {
    fn run(self, input: I) -> crate::Result<<I::Signature as InferenceSignature>::Output> {
        let inner = &mut self.inner.lock().unwrap();
        let ctx = input.make_run_context::<R>(inner);
        R::run(ctx)
            .and_then(TryInto::try_into)
            .map_err(|e| ErrorRepr::InferenceFailed(e).into())
    }
}

pub(crate) enum SessionOptionsByDomain {}

impl InferenceDomainAssociation for SessionOptionsByDomain {
    type Target<D: InferenceDomain> = EnumMap<D::Operation, InferenceSessionOptions>;
}

enum ModelInnerIdsByDomain {}

impl InferenceDomainAssociation for ModelInnerIdsByDomain {
    type Target<D: InferenceDomain> = BTreeMap<StyleId, ModelInnerId>;
}

struct SessionSetByDomain<R>(Infallible, PhantomData<fn() -> R>);

impl<R: InferenceRuntime> InferenceDomainAssociation for SessionSetByDomain<R> {
    type Target<D: InferenceDomain> = SessionSet<R, D>;
}

#[cfg(test)]
mod tests {
    use enum_map::enum_map;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::{
        infer::domains::{InferenceDomainGroupImpl, InferenceDomainMapImpl, TalkOperation},
        macros::tests::assert_debug_fmt_eq,
        synthesizer::InferenceRuntimeImpl,
        test_util::open_default_vvm_file,
    };

    use super::{super::InferenceSessionOptions, Status};

    #[rstest]
    #[case(true, 0)]
    #[case(true, 1)]
    #[case(true, 8)]
    #[case(false, 2)]
    #[case(false, 4)]
    #[case(false, 8)]
    #[case(false, 0)]
    fn status_new_works(#[case] use_gpu: bool, #[case] cpu_num_threads: u16) {
        let light_session_options = InferenceSessionOptions::new(cpu_num_threads, false);
        let heavy_session_options = InferenceSessionOptions::new(cpu_num_threads, use_gpu);
        let session_options = InferenceDomainMapImpl {
            talk: enum_map! {
                TalkOperation::PredictDuration
                | TalkOperation::PredictIntonation => light_session_options,
                TalkOperation::Decode => heavy_session_options,
            },
        };
        let status = Status::<InferenceRuntimeImpl, InferenceDomainGroupImpl>::new(session_options);

        assert_eq!(
            light_session_options,
            status.session_options.talk[TalkOperation::PredictDuration],
        );
        assert_eq!(
            light_session_options,
            status.session_options.talk[TalkOperation::PredictIntonation],
        );
        assert_eq!(
            heavy_session_options,
            status.session_options.talk[TalkOperation::Decode],
        );

        assert!(status.loaded_models.lock().unwrap().0.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn status_load_model_works() {
        let status =
            Status::<InferenceRuntimeImpl, InferenceDomainGroupImpl>::new(InferenceDomainMapImpl {
                talk: enum_map!(_ => InferenceSessionOptions::new(0, false)),
            });
        let model = &open_default_vvm_file().await;
        let model_bytes = &model.read_inference_models().await.unwrap();
        let result = status.insert_model(model.header(), model_bytes);
        assert_debug_fmt_eq!(Ok(()), result);
        assert_eq!(1, status.loaded_models.lock().unwrap().0.len());
    }

    #[rstest]
    #[tokio::test]
    async fn status_is_model_loaded_works() {
        let status =
            Status::<InferenceRuntimeImpl, InferenceDomainGroupImpl>::new(InferenceDomainMapImpl {
                talk: enum_map!(_ => InferenceSessionOptions::new(0, false)),
            });
        let vvm = open_default_vvm_file().await;
        let model_header = vvm.header();
        let model_bytes = &vvm.read_inference_models().await.unwrap();
        assert!(
            !status.is_loaded_model(&model_header.id),
            "model should  not be loaded"
        );
        let result = status.insert_model(model_header, model_bytes);
        assert_debug_fmt_eq!(Ok(()), result);
        assert!(
            status.is_loaded_model(&model_header.id),
            "model should be loaded",
        );
    }
}
