use super::*;
use itertools::iproduct;
use once_cell::sync::Lazy;
use onnxruntime::{
    environment::Environment,
    ndarray::{Ix0, Ix1, Ix2},
    session::{NdArray, Session},
    GraphOptimizationLevel, LoggingLevel,
};
use std::sync::Arc;
use std::{env, path::Path};
use tracing::error;

mod model_file;

cfg_if! {
    if #[cfg(not(feature="directml"))]{
        use onnxruntime::CudaProviderOptions;
    }
}
use std::collections::BTreeMap;

pub struct Status {
    loaded_models: std::sync::Mutex<LoadedModels>,
    light_session_options: SessionOptions, // 軽いモデルはこちらを使う
    heavy_session_options: SessionOptions, // 重いモデルはこちらを使う
}

#[derive(new, Getters)]
struct SessionOptions {
    cpu_num_threads: u16,
    use_gpu: bool,
}

#[derive(thiserror::Error, Debug)]
#[error("不正なモデルファイルです")]
struct DecryptModelError;

static ENVIRONMENT: Lazy<Environment> = Lazy::new(|| {
    cfg_if! {
        if #[cfg(debug_assertions)]{
            const LOGGING_LEVEL: LoggingLevel = LoggingLevel::Verbose;
        } else{
            const LOGGING_LEVEL: LoggingLevel = LoggingLevel::Warning;
        }
    }
    Environment::builder()
        .with_name(env!("CARGO_PKG_NAME"))
        .with_log_level(LOGGING_LEVEL)
        .build()
        .unwrap()
});

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

    fn new_session(
        &self,
        model: &[u8],
        session_options: &SessionOptions,
        path: impl AsRef<Path>,
    ) -> Result<Session<'static>> {
        self.new_session_from_bytes(|| model_file::decrypt(model), session_options)
            .map_err(|source| Error::LoadModel {
                path: path.as_ref().into(),
                source,
            })
    }

    fn new_session_from_bytes(
        &self,
        model_bytes: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        session_options: &SessionOptions,
    ) -> anyhow::Result<Session<'static>> {
        let session_builder = ENVIRONMENT
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::Basic)?
            .with_intra_op_num_threads(*session_options.cpu_num_threads() as i32)?
            .with_inter_op_num_threads(*session_options.cpu_num_threads() as i32)?;

        let session_builder = if *session_options.use_gpu() {
            cfg_if! {
                if #[cfg(feature = "directml")]{
                    session_builder
                        .with_disable_mem_pattern()?
                        .with_execution_mode(onnxruntime::ExecutionMode::ORT_SEQUENTIAL)?
                        .with_append_execution_provider_directml(0)?
                } else {
                    let options = CudaProviderOptions::default();
                    session_builder.with_append_execution_provider_cuda(options)?
                }
            }
        } else {
            session_builder
        };

        Ok(session_builder.with_model_from_memory(model_bytes()?)?)
    }

    pub fn validate_speaker_id(&self, style_id: StyleId) -> bool {
        self.is_loaded_model_by_style_id(style_id)
    }

    /// # Panics
    ///
    /// `self`が`model_id`を含んでいないとき、パニックする。
    pub async fn predict_duration_session_run(
        &self,
        model_id: &VoiceModelId,
        mut phoneme_vector_array: NdArray<i64, Ix1>,
        mut speaker_id_array: NdArray<i64, Ix1>,
    ) -> Result<Vec<f32>> {
        let predict_duration = self
            .loaded_models
            .lock()
            .unwrap()
            .predict_duration(model_id);

        tokio::task::spawn_blocking(move || {
            let mut predict_duration = predict_duration.lock().unwrap();

            let output_tensors = predict_duration
                .run(vec![&mut phoneme_vector_array, &mut speaker_id_array])
                .map_err(|_| Error::InferenceFailed)?;
            Ok(output_tensors[0].as_slice().unwrap().to_owned())
        })
        .await
        .unwrap()
    }

    /// # Panics
    ///
    /// `self`が`model_id`を含んでいないとき、パニックする。
    #[allow(clippy::too_many_arguments)]
    pub async fn predict_intonation_session_run(
        &self,
        model_id: &VoiceModelId,
        mut length_array: NdArray<i64, Ix0>,
        mut vowel_phoneme_vector_array: NdArray<i64, Ix1>,
        mut consonant_phoneme_vector_array: NdArray<i64, Ix1>,
        mut start_accent_vector_array: NdArray<i64, Ix1>,
        mut end_accent_vector_array: NdArray<i64, Ix1>,
        mut start_accent_phrase_vector_array: NdArray<i64, Ix1>,
        mut end_accent_phrase_vector_array: NdArray<i64, Ix1>,
        mut speaker_id_array: NdArray<i64, Ix1>,
    ) -> Result<Vec<f32>> {
        let predict_intonation = self
            .loaded_models
            .lock()
            .unwrap()
            .predict_intonation(model_id);

        tokio::task::spawn_blocking(move || {
            let mut predict_intonation = predict_intonation.lock().unwrap();

            let output_tensors = predict_intonation
                .run(vec![
                    &mut length_array,
                    &mut vowel_phoneme_vector_array,
                    &mut consonant_phoneme_vector_array,
                    &mut start_accent_vector_array,
                    &mut end_accent_vector_array,
                    &mut start_accent_phrase_vector_array,
                    &mut end_accent_phrase_vector_array,
                    &mut speaker_id_array,
                ])
                .map_err(|_| Error::InferenceFailed)?;
            Ok(output_tensors[0].as_slice().unwrap().to_owned())
        })
        .await
        .unwrap()
    }

    /// # Panics
    ///
    /// `self`が`model_id`を含んでいないとき、パニックする。
    pub async fn decode_session_run(
        &self,
        model_id: &VoiceModelId,
        mut f0_array: NdArray<f32, Ix2>,
        mut phoneme_array: NdArray<f32, Ix2>,
        mut speaker_id_array: NdArray<i64, Ix1>,
    ) -> Result<Vec<f32>> {
        let decode = self.loaded_models.lock().unwrap().decode(model_id);

        tokio::task::spawn_blocking(move || {
            let mut decode = decode.lock().unwrap();

            let output_tensors = decode
                .run(vec![
                    &mut f0_array,
                    &mut phoneme_array,
                    &mut speaker_id_array,
                ])
                .map_err(|_| Error::InferenceFailed)?;
            Ok(output_tensors[0].as_slice().unwrap().to_owned())
        })
        .await
        .unwrap()
    }
}

/// 読み込んだモデルの`Session`とそのメタ情報を保有し、追加/削除/取得の操作を提供する。
///
/// この構造体のメソッドは、すべて一瞬で完了すべきである。
#[derive(Default)]
struct LoadedModels(BTreeMap<VoiceModelId, LoadedModel>);

struct LoadedModel {
    model_inner_ids: BTreeMap<StyleId, ModelInnerId>,
    metas: VoiceModelMeta,
    session_set: SessionSet,
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
            .ok_or(Error::InvalidStyleId { style_id })?;

        let model_inner_id = *model_inner_ids
            .get(&style_id)
            .expect("`model_inner_ids` should contains all of the style IDs in the model");

        Ok((model_id.clone(), model_inner_id))
    }

    /// # Panics
    ///
    /// `self`が`model_id`を含んでいないとき、パニックする。
    fn predict_duration(
        &self,
        model_id: &VoiceModelId,
    ) -> Arc<std::sync::Mutex<AssertSend<Session<'static>>>> {
        let LoadedModel {
            session_set: SessionSet {
                predict_duration, ..
            },
            ..
        } = &self.0[model_id];
        predict_duration.clone()
    }

    /// # Panics
    ///
    /// `self`が`model_id`を含んでいないとき、パニックする。
    fn predict_intonation(
        &self,
        model_id: &VoiceModelId,
    ) -> Arc<std::sync::Mutex<AssertSend<Session<'static>>>> {
        let LoadedModel {
            session_set: SessionSet {
                predict_intonation, ..
            },
            ..
        } = &self.0[model_id];
        predict_intonation.clone()
    }

    /// # Panics
    ///
    /// `self`が`model_id`を含んでいないとき、パニックする。
    fn decode(
        &self,
        model_id: &VoiceModelId,
    ) -> Arc<std::sync::Mutex<AssertSend<Session<'static>>>> {
        let LoadedModel {
            session_set: SessionSet { decode, .. },
            ..
        } = &self.0[model_id];
        decode.clone()
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
    fn ensure_acceptable(&self, model: &VoiceModel) -> Result<()> {
        let loaded = self.styles();
        let external = model.metas().iter().flat_map(|speaker| speaker.styles());

        if self.0.contains_key(model.id())
            || iproduct!(loaded, external).any(|(loaded, external)| loaded.id() == external.id())
        {
            return Err(Error::AlreadyLoadedModel {
                path: model.path().clone(),
            });
        }
        Ok(())
    }

    fn insert(
        &mut self,
        model: &VoiceModel,
        predict_duration: Session<'static>,
        predict_intonation: Session<'static>,
        decode: Session<'static>,
    ) -> Result<()> {
        self.ensure_acceptable(model)?;

        let prev = self.0.insert(
            model.id().clone(),
            LoadedModel {
                model_inner_ids: model.model_inner_ids(),
                metas: model.metas().clone(),
                session_set: SessionSet {
                    predict_duration: Arc::new(std::sync::Mutex::new(predict_duration.into())),
                    predict_intonation: Arc::new(std::sync::Mutex::new(predict_intonation.into())),
                    decode: Arc::new(std::sync::Mutex::new(decode.into())),
                },
            },
        );
        assert!(prev.is_none());
        Ok(())
    }

    fn remove(&mut self, model_id: &VoiceModelId) -> Result<()> {
        if self.0.remove(model_id).is_none() {
            return Err(Error::UnloadedModel {
                model_id: model_id.clone(),
            });
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

struct SessionSet {
    predict_duration: Arc<std::sync::Mutex<AssertSend<Session<'static>>>>,
    predict_intonation: Arc<std::sync::Mutex<AssertSend<Session<'static>>>>,
    decode: Arc<std::sync::Mutex<AssertSend<Session<'static>>>>,
}

// FIXME: 以下のことをちゃんと確認した後、onnxruntime-rs側で`Session`が`Send`であると宣言する。
// https://github.com/VOICEVOX/voicevox_core/issues/307#issuecomment-1276184614

use self::assert_send::AssertSend;

mod assert_send {
    use std::ops::{Deref, DerefMut};

    use onnxruntime::session::Session;

    pub(super) struct AssertSend<T>(T);

    impl From<Session<'static>> for AssertSend<Session<'static>> {
        fn from(session: Session<'static>) -> Self {
            Self(session)
        }
    }

    impl<T> Deref for AssertSend<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> DerefMut for AssertSend<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    // SAFETY: `Session` is probably "send"able.
    #[allow(unsafe_code)]
    unsafe impl<T> Send for AssertSend<T> {}
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
