use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
    sync::Arc,
};

use educe::Educe;
use enum_map::EnumMap;
use itertools::iproduct;

use crate::{
    error::{ErrorRepr, LoadModelError, LoadModelErrorKind, LoadModelResult},
    manifest::ModelInnerId,
    metas::{SpeakerMeta, StyleId, StyleMeta, VoiceModelMeta},
    voice_model::{VoiceModel, VoiceModelId},
    Result,
};

use super::{
    model_file, InferenceGroup, InferenceInputSignature, InferenceRuntime, InferenceSessionOptions,
    InferenceSignature,
};

pub(crate) struct Status<R: InferenceRuntime, G: InferenceGroup> {
    loaded_models: std::sync::Mutex<LoadedModels<R, G>>,
    session_options: EnumMap<G, InferenceSessionOptions>,
}

impl<R: InferenceRuntime, G: InferenceGroup> Status<R, G> {
    pub fn new(session_options: EnumMap<G, InferenceSessionOptions>) -> Self {
        Self {
            loaded_models: Default::default(),
            session_options,
        }
    }

    pub async fn load_model(
        &self,
        model: &VoiceModel,
        model_bytes: &EnumMap<G, Vec<u8>>,
    ) -> Result<()> {
        self.loaded_models
            .lock()
            .unwrap()
            .ensure_acceptable(model)?;

        let session_set =
            SessionSet::new(model_bytes, &self.session_options).map_err(|source| {
                LoadModelError {
                    path: model.path().clone(),
                    context: LoadModelErrorKind::InvalidModelData,
                    source: Some(source),
                }
            })?;

        self.loaded_models
            .lock()
            .unwrap()
            .insert(model, session_set)?;
        Ok(())
    }

    pub fn unload_model(&self, voice_model_id: &VoiceModelId) -> Result<()> {
        self.loaded_models.lock().unwrap().remove(voice_model_id)
    }

    pub fn metas(&self) -> VoiceModelMeta {
        self.loaded_models.lock().unwrap().metas()
    }

    pub(crate) fn ids_for(&self, style_id: StyleId) -> Result<(VoiceModelId, ModelInnerId)> {
        self.loaded_models.lock().unwrap().ids_for(style_id)
    }

    pub fn is_loaded_model(&self, voice_model_id: &VoiceModelId) -> bool {
        self.loaded_models
            .lock()
            .unwrap()
            .contains_voice_model(voice_model_id)
    }

    pub fn is_loaded_model_by_style_id(&self, style_id: StyleId) -> bool {
        self.loaded_models.lock().unwrap().contains_style(style_id)
    }

    pub fn validate_speaker_id(&self, style_id: StyleId) -> bool {
        self.is_loaded_model_by_style_id(style_id)
    }

    /// # Panics
    ///
    /// `self`が`model_id`を含んでいないとき、パニックする。
    pub(crate) async fn run_session<I>(
        &self,
        model_id: &VoiceModelId,
        input: I,
    ) -> Result<<I::Signature as InferenceSignature>::Output>
    where
        I: InferenceInputSignature,
        I::Signature: InferenceSignature<Group = G>,
    {
        let sess = self.loaded_models.lock().unwrap().get(model_id);

        tokio::task::spawn_blocking(move || sess.run(input))
            .await
            .unwrap()
    }
}

/// 読み込んだモデルの`Session`とそのメタ情報を保有し、追加/削除/取得の操作を提供する。
///
/// この構造体のメソッドは、すべて一瞬で完了すべきである。
#[derive(Educe)]
#[educe(Default(bound = "R: InferenceRuntime, G: InferenceGroup"))]
struct LoadedModels<R: InferenceRuntime, G: InferenceGroup>(
    BTreeMap<VoiceModelId, LoadedModel<R, G>>,
);

struct LoadedModel<R: InferenceRuntime, G: InferenceGroup> {
    model_inner_ids: BTreeMap<StyleId, ModelInnerId>,
    metas: VoiceModelMeta,
    session_set: SessionSet<R, G>,
}

impl<R: InferenceRuntime, G: InferenceGroup> LoadedModels<R, G> {
    fn metas(&self) -> VoiceModelMeta {
        self.0
            .values()
            .flat_map(|LoadedModel { metas, .. }| metas)
            .cloned()
            .collect()
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
        I::Signature: InferenceSignature<Group = G>,
    {
        self.0[model_id].session_set.get()
    }

    fn contains_voice_model(&self, model_id: &VoiceModelId) -> bool {
        self.0.contains_key(model_id)
    }

    fn contains_style(&self, style_id: StyleId) -> bool {
        self.styles().any(|style| *style.id() == style_id)
    }

    /// 与えられた`VoiceModel`を受け入れ可能かをチェックする。
    ///
    /// # Errors
    ///
    /// 音声モデルIDかスタイルIDが`model`と重複するとき、エラーを返す。
    fn ensure_acceptable(&self, model: &VoiceModel) -> LoadModelResult<()> {
        let loaded = self.styles();
        let external = model.metas().iter().flat_map(|speaker| speaker.styles());

        let error = |context| LoadModelError {
            path: model.path().clone(),
            context,
            source: None,
        };

        if self.0.contains_key(model.id()) {
            return Err(error(LoadModelErrorKind::ModelAlreadyLoaded {
                id: model.id().clone(),
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

    fn insert(&mut self, model: &VoiceModel, session_set: SessionSet<R, G>) -> Result<()> {
        self.ensure_acceptable(model)?;

        let prev = self.0.insert(
            model.id().clone(),
            LoadedModel {
                model_inner_ids: model.model_inner_ids(),
                metas: model.metas().clone(),
                session_set,
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

    fn styles(&self) -> impl Iterator<Item = &StyleMeta> {
        self.0
            .values()
            .flat_map(|LoadedModel { metas, .. }| metas)
            .flat_map(|speaker| speaker.styles())
    }
}

struct SessionSet<R: InferenceRuntime, G: InferenceGroup>(
    EnumMap<G, Arc<std::sync::Mutex<R::Session>>>,
);

impl<R: InferenceRuntime, G: InferenceGroup> SessionSet<R, G> {
    fn new(
        model_bytes: &EnumMap<G, Vec<u8>>,
        options: &EnumMap<G, InferenceSessionOptions>,
    ) -> anyhow::Result<Self> {
        let mut sessions = model_bytes
            .iter()
            .map(|(k, m)| {
                let sess = R::new_session(|| model_file::decrypt(m), options[k])?;
                Ok((k.into_usize(), std::sync::Mutex::new(sess).into()))
            })
            .collect::<anyhow::Result<HashMap<_, _>>>()?;

        Ok(Self(EnumMap::<G, _>::from_fn(|k| {
            sessions.remove(&k.into_usize()).expect("should exist")
        })))
    }
}

impl<R: InferenceRuntime, G: InferenceGroup> SessionSet<R, G> {
    fn get<I>(&self) -> SessionCell<R, I>
    where
        I: InferenceInputSignature,
        I::Signature: InferenceSignature<Group = G>,
    {
        SessionCell {
            inner: self.0[I::Signature::KIND].clone(),
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

#[cfg(test)]
mod tests {
    use enum_map::enum_map;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::{
        infer::signatures::InferenceKind, macros::tests::assert_debug_fmt_eq,
        synthesizer::InferenceRuntimeImpl, test_util::open_default_vvm_file,
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
        let session_options = enum_map! {
            InferenceKind::PredictDuration
            | InferenceKind::PredictIntonation => light_session_options,
            InferenceKind::Decode => heavy_session_options,
        };
        let status = Status::<InferenceRuntimeImpl, InferenceKind>::new(session_options);

        assert_eq!(
            light_session_options,
            status.session_options[InferenceKind::PredictDuration],
        );
        assert_eq!(
            light_session_options,
            status.session_options[InferenceKind::PredictIntonation],
        );
        assert_eq!(
            heavy_session_options,
            status.session_options[InferenceKind::Decode],
        );

        assert!(status.loaded_models.lock().unwrap().0.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn status_load_model_works() {
        let status = Status::<InferenceRuntimeImpl, InferenceKind>::new(
            enum_map!(_ => InferenceSessionOptions::new(0, false)),
        );
        let model = &open_default_vvm_file().await;
        let model_bytes = &model.read_inference_models().await.unwrap();
        let result = status.load_model(model, model_bytes).await;
        assert_debug_fmt_eq!(Ok(()), result);
        assert_eq!(1, status.loaded_models.lock().unwrap().0.len());
    }

    #[rstest]
    #[tokio::test]
    async fn status_is_model_loaded_works() {
        let status = Status::<InferenceRuntimeImpl, InferenceKind>::new(
            enum_map!(_ => InferenceSessionOptions::new(0, false)),
        );
        let vvm = open_default_vvm_file().await;
        let model_bytes = &vvm.read_inference_models().await.unwrap();
        assert!(
            !status.is_loaded_model(vvm.id()),
            "model should  not be loaded"
        );
        let result = status.load_model(&vvm, model_bytes).await;
        assert_debug_fmt_eq!(Ok(()), result);
        assert!(status.is_loaded_model(vvm.id()), "model should be loaded");
    }
}
