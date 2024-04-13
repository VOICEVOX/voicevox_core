use std::{any, convert::Infallible, marker::PhantomData};

use educe::Educe;
use enum_map::EnumMap;
use indexmap::IndexMap;
use itertools::{iproduct, Itertools as _};

use crate::{
    error::{ErrorRepr, LoadModelError, LoadModelErrorKind, LoadModelResult},
    infer::{
        domains::{InferenceDomainGroupImpl, InferenceDomainMapImpl, TalkDomain},
        session_set::{SessionCell, SessionSet},
        ForAllInferenceDomain, InferenceDomain, InferenceDomainMapValueProjection,
        InferenceInputSignature, InferenceRuntime, InferenceSessionOptions, InferenceSignature,
    },
    manifest::{ModelInnerId, StyleIdToModelInnerId},
    metas::{self, SpeakerMeta, StyleId, StyleMeta, VoiceModelMeta},
    voice_model::{ModelBytesByInferenceDomain, VoiceModelHeader, VoiceModelId},
    Result, StyleType,
};

pub(crate) struct Status<R: InferenceRuntime> {
    loaded_models: std::sync::Mutex<LoadedModels<R>>,
    session_options: InferenceDomainMapImpl<SessionOptionsByDomain>,
}

impl<R: InferenceRuntime> Status<R> {
    pub(crate) fn new(session_options: InferenceDomainMapImpl<SessionOptionsByDomain>) -> Self {
        Self {
            loaded_models: Default::default(),
            session_options,
        }
    }

    pub(crate) fn insert_model(
        &self,
        model_header: &VoiceModelHeader,
        model_bytes: &InferenceDomainMapImpl<
            Option<(
                ForAllInferenceDomain<StyleIdToModelInnerId>,
                ModelBytesByInferenceDomain,
            )>,
        >,
    ) -> Result<()> {
        self.loaded_models
            .lock()
            .unwrap()
            .ensure_acceptable(model_header, model_bytes.each_is_some())?;

        let session_set = (|| {
            let talk = model_bytes
                .talk
                .as_ref()
                .map(|(model_inner_ids, model_bytes)| {
                    let session_set = SessionSet::new(model_bytes, &self.session_options.talk)?;
                    Ok::<_, anyhow::Error>((model_inner_ids.clone(), session_set))
                })
                .transpose()?;
            Ok(InferenceDomainMapImpl { talk })
        })()
        .map_err(|source| LoadModelError {
            path: model_header.path.clone(),
            context: LoadModelErrorKind::InvalidModelData,
            source: Some(source),
        })?;

        self.loaded_models
            .lock()
            .unwrap()
            .insert(model_header, session_set)?;
        Ok(())
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
    pub(crate) fn ids_for<D: InferenceDomain<Group = InferenceDomainGroupImpl>>(
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
        <I::Signature as InferenceSignature>::Domain:
            InferenceDomain<Group = InferenceDomainGroupImpl>,
    {
        let sess = self.loaded_models.lock().unwrap().get(model_id);
        sess.run(input)
    }
}

/// 読み込んだモデルの`Session`とそのメタ情報を保有し、追加/削除/取得の操作を提供する。
///
/// この構造体のメソッドは、すべて一瞬で完了すべきである。
#[derive(Educe)]
#[educe(Default(bound = "R: InferenceRuntime"))]
struct LoadedModels<R: InferenceRuntime>(IndexMap<VoiceModelId, LoadedModel<R>>);

struct LoadedModel<R: InferenceRuntime> {
    metas: VoiceModelMeta,
    #[allow(clippy::type_complexity)]
    by_domain: InferenceDomainMapImpl<
        Option<(
            ForAllInferenceDomain<StyleIdToModelInnerId>,
            SessionSetByDomain<R>,
        )>,
    >,
}

impl<R: InferenceRuntime> LoadedModels<R> {
    fn metas(&self) -> VoiceModelMeta {
        metas::merge(self.0.values().flat_map(|LoadedModel { metas, .. }| metas))
    }

    fn ids_for<D: InferenceDomain<Group = InferenceDomainGroupImpl>>(
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
        <I::Signature as InferenceSignature>::Domain:
            InferenceDomain<Group = InferenceDomainGroupImpl>,
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
                         `ensure_acceptable` and `ids_for`)",
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
        existences: InferenceDomainMapImpl<ForAllInferenceDomain<bool>>,
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
        if let Some(style_type) = external
            .clone()
            .map(StyleMeta::r#type)
            .copied()
            .unique()
            .find(|&style_type| !existences.matches(style_type))
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
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn insert(
        &mut self,
        model_header: &VoiceModelHeader,
        by_domain: InferenceDomainMapImpl<
            Option<(
                ForAllInferenceDomain<StyleIdToModelInnerId>,
                SessionSetByDomain<R>,
            )>,
        >,
    ) -> Result<()> {
        self.ensure_acceptable(model_header, by_domain.each_is_some())?;

        let prev = self.0.insert(
            model_header.id.clone(),
            LoadedModel {
                metas: model_header.metas.clone(),
                by_domain,
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

pub(crate) enum SessionOptionsByDomain {}

impl InferenceDomainMapValueProjection for SessionOptionsByDomain {
    type Target<D: InferenceDomain> = EnumMap<D::Operation, InferenceSessionOptions>;
}

struct SessionSetByDomain<R>(Infallible, PhantomData<fn() -> R>);

impl<R: InferenceRuntime> InferenceDomainMapValueProjection for SessionSetByDomain<R> {
    type Target<D: InferenceDomain> = SessionSet<R, D>;
}

impl<V: InferenceDomainMapValueProjection> InferenceDomainMapImpl<Option<V>> {
    fn each_is_some(&self) -> InferenceDomainMapImpl<ForAllInferenceDomain<bool>> {
        InferenceDomainMapImpl {
            talk: self.talk.is_some(),
        }
    }
}

impl InferenceDomainMapImpl<ForAllInferenceDomain<bool>> {
    fn matches(&self, style_type: StyleType) -> bool {
        let Self { talk } = *self;
        talk && TalkDomain::style_types().contains(&style_type)
    }
}

#[cfg(test)]
mod tests {
    use enum_map::enum_map;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::{
        infer::{
            domains::{InferenceDomainMapImpl, TalkOperation},
            InferenceSessionOptions,
        },
        macros::tests::assert_debug_fmt_eq,
        synthesizer::InferenceRuntimeImpl,
        test_util::open_default_vvm_file,
    };

    use super::Status;

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
        let status = Status::<InferenceRuntimeImpl>::new(session_options);

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
        let status = Status::<InferenceRuntimeImpl>::new(InferenceDomainMapImpl {
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
        let status = Status::<InferenceRuntimeImpl>::new(InferenceDomainMapImpl {
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
