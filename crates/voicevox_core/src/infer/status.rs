use std::{
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
    voice_model::{InferenceModelsByInferenceDomain, VoiceModelHeader, VoiceModelId},
    Result,
};

use super::{
    model_file, InferenceDomain, InferenceDomainMap as _, InferenceInputSignature,
    InferenceRuntime, InferenceSessionOptions, InferenceSignature, Optional,
};

pub(crate) struct Status<R: InferenceRuntime, S: InferenceDomainGroup> {
    loaded_models: std::sync::Mutex<LoadedModels<R, S>>,
    session_options: S::Map<SessionOptionsByDomain>,
}

impl<R: InferenceRuntime, S: InferenceDomainGroup> Status<R, S> {
    pub(crate) fn new(session_options: S::Map<SessionOptionsByDomain>) -> Self {
        Self {
            loaded_models: Default::default(),
            session_options,
        }
    }

    pub(crate) fn insert_model(
        &self,
        model_header: &VoiceModelHeader,
        model_bytes: &S::Map<Optional<InferenceModelsByInferenceDomain>>,
    ) -> Result<()> {
        self.loaded_models
            .lock()
            .unwrap()
            .ensure_acceptable(model_header)?;

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

        struct CreateSessionSet<'a, R, S: InferenceDomainGroup> {
            session_options: &'a S::Map<SessionOptionsByDomain>,
            marker: PhantomData<fn() -> R>,
        }

        impl<R: InferenceRuntime, S: InferenceDomainGroup>
            ConvertInferenceDomainAssociationTarget<
                S,
                Optional<InferenceModelsByInferenceDomain>,
                Optional<SessionSetByDomain<R>>,
                anyhow::Error,
            > for CreateSessionSet<'_, R, S>
        {
            fn try_ref_map<D: InferenceDomain<Group = S>>(
                &self,
                model_bytes: &<Optional<InferenceModelsByInferenceDomain> as InferenceDomainAssociation>::Target<D>,
            ) -> anyhow::Result<
                <Optional<SessionSetByDomain<R>> as InferenceDomainAssociation>::Target<D>,
            > {
                model_bytes
                    .as_ref()
                    .map(|model_bytes| SessionSet::new(model_bytes, D::visit(self.session_options)))
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

    pub(crate) fn ids_for(&self, style_id: StyleId) -> Result<(VoiceModelId, ModelInnerId)> {
        self.loaded_models.lock().unwrap().ids_for(style_id)
    }

    pub(crate) fn is_loaded_model(&self, voice_model_id: &VoiceModelId) -> bool {
        self.loaded_models
            .lock()
            .unwrap()
            .contains_voice_model(voice_model_id)
    }

    pub(crate) fn is_loaded_model_by_style_id(&self, style_id: StyleId) -> bool {
        self.loaded_models.lock().unwrap().contains_style(style_id)
    }

    pub(crate) fn validate_speaker_id(&self, style_id: StyleId) -> bool {
        self.is_loaded_model_by_style_id(style_id)
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
        <I::Signature as InferenceSignature>::Domain: InferenceDomain<Group = S>,
    {
        let sess = self.loaded_models.lock().unwrap().get(model_id);
        sess.run(input)
    }
}

/// 読み込んだモデルの`Session`とそのメタ情報を保有し、追加/削除/取得の操作を提供する。
///
/// この構造体のメソッドは、すべて一瞬で完了すべきである。
#[derive(Educe)]
#[educe(Default(bound = "R: InferenceRuntime, S: InferenceDomainGroup"))]
struct LoadedModels<R: InferenceRuntime, S: InferenceDomainGroup>(
    IndexMap<VoiceModelId, LoadedModel<R, S>>,
);

struct LoadedModel<R: InferenceRuntime, S: InferenceDomainGroup> {
    model_inner_ids: BTreeMap<StyleId, ModelInnerId>,
    metas: VoiceModelMeta,
    session_sets: S::Map<Optional<SessionSetByDomain<R>>>,
}

impl<R: InferenceRuntime, S: InferenceDomainGroup> LoadedModels<R, S> {
    fn metas(&self) -> VoiceModelMeta {
        metas::merge(self.0.values().flat_map(|LoadedModel { metas, .. }| metas))
    }

    fn ids_for(&self, style_id: StyleId) -> Result<(VoiceModelId, ModelInnerId)> {
        let (
            model_id,
            LoadedModel {
                model_inner_ids, ..
            },
        ) = self
            .0
            .iter()
            .find(|(_, LoadedModel { metas, .. })| {
                metas
                    .iter()
                    .flat_map(SpeakerMeta::styles)
                    .any(|style| *style.id() == style_id)
            })
            .ok_or(ErrorRepr::StyleNotFound { style_id })?;

        let model_inner_id = *model_inner_ids
            .get(&style_id)
            .expect("`model_inner_ids` should contains all of the style IDs in the model");

        Ok((model_id.clone(), model_inner_id))
    }

    /// # Panics
    ///
    /// `self`が`model_id`を含んでいないとき、パニックする。
    fn get<I>(&self, model_id: &VoiceModelId) -> SessionCell<R, I>
    where
        I: InferenceInputSignature,
        <I::Signature as InferenceSignature>::Domain: InferenceDomain<Group = S>,
    {
        <I::Signature as InferenceSignature>::Domain::visit(&self.0[model_id].session_sets)
            .as_ref()
            .unwrap_or_else(|| todo!("`ensure_acceptable`で検査する"))
            .get()
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
    /// - 音声モデルIDかスタイルIDが`model_header`と重複するとき
    fn ensure_acceptable(&self, model_header: &VoiceModelHeader) -> LoadModelResult<()> {
        let error = |context| LoadModelError {
            path: model_header.path.clone(),
            context,
            source: None,
        };

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
        if self.0.contains_key(&model_header.id) {
            return Err(error(LoadModelErrorKind::ModelAlreadyLoaded {
                id: model_header.id.clone(),
            }));
        }
        if let Some((style, _)) =
            iproduct!(loaded, external).find(|(loaded, external)| loaded.id() == external.id())
        {
            return Err(error(LoadModelErrorKind::StyleAlreadyLoaded {
                id: *style.id(),
            }));
        }
        Ok(())
    }

    fn insert(
        &mut self,
        model_header: &VoiceModelHeader,
        session_sets: S::Map<Optional<SessionSetByDomain<R>>>,
    ) -> Result<()> {
        self.ensure_acceptable(model_header)?;

        let prev = self.0.insert(
            model_header.id.clone(),
            LoadedModel {
                model_inner_ids: model_header.model_inner_ids(),
                metas: model_header.metas.clone(),
                session_sets,
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
