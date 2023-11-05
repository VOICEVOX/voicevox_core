use super::*;
use crate::infer::{
    runtimes::Onnxruntime,
    signatures::{Decode, PredictDuration, PredictIntonation, SessionSet},
    DecryptModelError, Output, SessionOptions, Signature, TypedSession,
};
use derive_more::Index;
use itertools::iproduct;
use std::path::Path;
use std::sync::Arc;

mod model_file;

use std::collections::BTreeMap;

pub struct Status {
    loaded_models: std::sync::Mutex<LoadedModels>,
    light_session_options: SessionOptions, // 軽いモデルはこちらを使う
    heavy_session_options: SessionOptions, // 重いモデルはこちらを使う
}

impl Status {
    pub fn new(use_gpu: bool, cpu_num_threads: u16) -> Self {
        Self {
            loaded_models: Default::default(),
            light_session_options: SessionOptions::new(cpu_num_threads, false),
            heavy_session_options: SessionOptions::new(cpu_num_threads, use_gpu),
        }
    }

    pub async fn load_model(&self, model: &VoiceModel) -> Result<()> {
        self.loaded_models
            .lock()
            .unwrap()
            .ensure_acceptable(model)?;

        let models = model.read_inference_models().await?;

        let predict_duration_session = self.new_session(
            models.predict_duration_model(),
            &self.light_session_options,
            model.path(),
        )?;
        let predict_intonation_session = self.new_session(
            models.predict_intonation_model(),
            &self.light_session_options,
            model.path(),
        )?;
        let decode_model = self.new_session(
            models.decode_model(),
            &self.heavy_session_options,
            model.path(),
        )?;

        self.loaded_models.lock().unwrap().insert(
            model,
            predict_duration_session,
            predict_intonation_session,
            decode_model,
        )?;
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

    fn new_session<S: Signature>(
        &self,
        model: &[u8],
        session_options: &SessionOptions,
        path: impl AsRef<Path>,
    ) -> LoadModelResult<TypedSession<Onnxruntime, S>> {
        TypedSession::<Onnxruntime, S>::new(|| model_file::decrypt(model), *session_options)
            .map_err(|source| LoadModelError {
                path: path.as_ref().to_owned(),
                context: LoadModelErrorKind::InvalidModelData,
                source: Some(source),
            })
    }

    pub fn validate_speaker_id(&self, style_id: StyleId) -> bool {
        self.is_loaded_model_by_style_id(style_id)
    }

    /// # Panics
    ///
    /// `self`が`model_id`を含んでいないとき、パニックする。
    pub(crate) async fn run_session<S>(
        &self,
        model_id: &VoiceModelId,
        input: S,
    ) -> Result<S::Output>
    where
        S: Signature,
        for<'a> &'a S::SessionSet<Onnxruntime>: From<&'a SessionSet<Onnxruntime>>,
        S::Output: Output<Onnxruntime>,
    {
        let sess = S::get_session::<Onnxruntime>(
            (&self.loaded_models.lock().unwrap()[model_id].session_set).into(),
        )
        .clone();

        tokio::task::spawn_blocking(move || {
            let mut sess = sess.lock().unwrap();
            sess.run(input)
                .map_err(|e| ErrorRepr::InferenceFailed(e).into())
        })
        .await
        .unwrap()
    }
}

/// 読み込んだモデルの`Session`とそのメタ情報を保有し、追加/削除/取得の操作を提供する。
///
/// この構造体のメソッドは、すべて一瞬で完了すべきである。
#[derive(Default, Index)]
struct LoadedModels(BTreeMap<VoiceModelId, LoadedModel>);

struct LoadedModel {
    model_inner_ids: BTreeMap<StyleId, ModelInnerId>,
    metas: VoiceModelMeta,
    session_set: SessionSet<Onnxruntime>,
}

impl LoadedModels {
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

    fn insert(
        &mut self,
        model: &VoiceModel,
        predict_duration: TypedSession<Onnxruntime, PredictDuration>,
        predict_intonation: TypedSession<Onnxruntime, PredictIntonation>,
        decode: TypedSession<Onnxruntime, Decode>,
    ) -> Result<()> {
        self.ensure_acceptable(model)?;

        let prev = self.0.insert(
            model.id().clone(),
            LoadedModel {
                model_inner_ids: model.model_inner_ids(),
                metas: model.metas().clone(),
                session_set: SessionSet {
                    predict_duration: Arc::new(std::sync::Mutex::new(predict_duration)),
                    predict_intonation: Arc::new(std::sync::Mutex::new(predict_intonation)),
                    decode: Arc::new(std::sync::Mutex::new(decode)),
                },
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::macros::tests::assert_debug_fmt_eq;
    use pretty_assertions::assert_eq;

    #[rstest]
    #[case(true, 0)]
    #[case(true, 1)]
    #[case(true, 8)]
    #[case(false, 2)]
    #[case(false, 4)]
    #[case(false, 8)]
    #[case(false, 0)]
    fn status_new_works(#[case] use_gpu: bool, #[case] cpu_num_threads: u16) {
        let status = Status::new(use_gpu, cpu_num_threads);
        assert_eq!(false, status.light_session_options.use_gpu);
        assert_eq!(use_gpu, status.heavy_session_options.use_gpu);
        assert_eq!(
            cpu_num_threads,
            status.light_session_options.cpu_num_threads
        );
        assert_eq!(
            cpu_num_threads,
            status.heavy_session_options.cpu_num_threads
        );
        assert!(status.loaded_models.lock().unwrap().0.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn status_load_model_works() {
        let status = Status::new(false, 0);
        let result = status.load_model(&open_default_vvm_file().await).await;
        assert_debug_fmt_eq!(Ok(()), result);
        assert_eq!(1, status.loaded_models.lock().unwrap().0.len());
    }

    #[rstest]
    #[tokio::test]
    async fn status_is_model_loaded_works() {
        let status = Status::new(false, 0);
        let vvm = open_default_vvm_file().await;
        assert!(
            !status.is_loaded_model(vvm.id()),
            "model should  not be loaded"
        );
        let result = status.load_model(&vvm).await;
        assert_debug_fmt_eq!(Ok(()), result);
        assert!(status.is_loaded_model(vvm.id()), "model should be loaded");
    }
}
