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
            .ensure_not_contains(model)?;

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
            LightOrtSessions {
                predict_duration: predict_duration_session.into(),
                predict_intonation: predict_intonation_session.into(),
            },
            HeavyOrtSession {
                decode: decode_model.into(),
            },
        )?;
        Ok(())
    }

    pub fn unload_model(&self, voice_model_id: &VoiceModelId) -> Result<()> {
        self.loaded_models.lock().unwrap().remove(voice_model_id)
    }

    pub fn metas(&self) -> VoiceModelMeta {
        self.loaded_models.lock().unwrap().metas()
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

    pub async fn predict_duration_session_run(
        &self,
        style_id: StyleId,
        mut phoneme_vector_array: NdArray<i64, Ix1>,
        mut speaker_id_array: NdArray<i64, Ix1>,
    ) -> Result<Vec<f32>> {
        let light_sessions = self
            .loaded_models
            .lock()
            .unwrap()
            .light_sessions(style_id)?;

        tokio::task::spawn_blocking(move || {
            let LightOrtSessions {
                predict_duration, ..
            } = &mut *light_sessions.lock().unwrap();

            let output_tensors = predict_duration
                .get_mut()
                .run(vec![&mut phoneme_vector_array, &mut speaker_id_array])
                .map_err(|_| Error::InferenceFailed)?;
            Ok(output_tensors[0].as_slice().unwrap().to_owned())
        })
        .await
        .unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn predict_intonation_session_run(
        &self,
        style_id: StyleId,
        mut length_array: NdArray<i64, Ix0>,
        mut vowel_phoneme_vector_array: NdArray<i64, Ix1>,
        mut consonant_phoneme_vector_array: NdArray<i64, Ix1>,
        mut start_accent_vector_array: NdArray<i64, Ix1>,
        mut end_accent_vector_array: NdArray<i64, Ix1>,
        mut start_accent_phrase_vector_array: NdArray<i64, Ix1>,
        mut end_accent_phrase_vector_array: NdArray<i64, Ix1>,
        mut speaker_id_array: NdArray<i64, Ix1>,
    ) -> Result<Vec<f32>> {
        let light_sessions = self
            .loaded_models
            .lock()
            .unwrap()
            .light_sessions(style_id)?;

        tokio::task::spawn_blocking(move || {
            let LightOrtSessions {
                predict_intonation, ..
            } = &mut *light_sessions.lock().unwrap();

            let output_tensors = predict_intonation
                .get_mut()
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

    pub async fn decode_session_run(
        &self,
        style_id: StyleId,
        mut f0_array: NdArray<f32, Ix2>,
        mut phoneme_array: NdArray<f32, Ix2>,
        mut speaker_id_array: NdArray<i64, Ix1>,
    ) -> Result<Vec<f32>> {
        let heavy_session = self.loaded_models.lock().unwrap().heavy_session(style_id)?;

        tokio::task::spawn_blocking(move || {
            let HeavyOrtSession { decode } = &mut *heavy_session.lock().unwrap();

            let output_tensors = decode
                .get_mut()
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

#[derive(Default)]
struct LoadedModels(BTreeMap<VoiceModelId, LoadedModel>);

struct LoadedModel {
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

    fn light_sessions(
        &mut self,
        style_id: StyleId,
    ) -> Result<Arc<std::sync::Mutex<LightOrtSessions>>> {
        let LoadedModel { session_set, .. } = self.find_loaded_voice_model(style_id)?;
        Ok(session_set.get_light())
    }

    fn heavy_session(
        &mut self,
        style_id: StyleId,
    ) -> Result<Arc<std::sync::Mutex<HeavyOrtSession>>> {
        let LoadedModel { session_set, .. } = self.find_loaded_voice_model(style_id)?;
        Ok(session_set.get_heavy())
    }

    fn contains_voice_model(&self, model_id: &VoiceModelId) -> bool {
        self.0.contains_key(model_id)
    }

    fn contains_style(&self, style_id: StyleId) -> bool {
        self.styles().any(|style| *style.id() == style_id)
    }

    fn ensure_not_contains(&self, model: &VoiceModel) -> Result<()> {
        let loaded = self.styles();
        let external = model.metas().iter().flat_map(|speaker| speaker.styles());

        if iproduct!(loaded, external).any(|(loaded, external)| loaded.id() == external.id()) {
            return Err(Error::AlreadyLoadedModel {
                path: model.path().clone(),
            });
        }
        Ok(())
    }

    fn insert(
        &mut self,
        model: &VoiceModel,
        light_sessions: LightOrtSessions,
        heavy_session: HeavyOrtSession,
    ) -> Result<()> {
        self.ensure_not_contains(model)?;

        let prev = self.0.insert(
            model.id().clone(),
            LoadedModel {
                metas: model.metas().clone(),
                session_set: SessionSet::new(light_sessions, heavy_session),
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

    fn find_loaded_voice_model(&mut self, style_id: StyleId) -> Result<&mut LoadedModel> {
        self.0
            .values_mut()
            .find(|LoadedModel { metas, .. }| {
                metas
                    .iter()
                    .flat_map(|speaker| speaker.styles())
                    .any(|style| *style.id() == style_id)
            })
            .ok_or(Error::InvalidStyleId { style_id })
    }

    fn styles(&self) -> impl Iterator<Item = &StyleMeta> {
        self.0
            .values()
            .flat_map(|LoadedModel { metas, .. }| metas)
            .flat_map(|speaker| speaker.styles())
    }
}

struct SessionSet {
    light: Arc<std::sync::Mutex<LightOrtSessions>>,
    heavy: Arc<std::sync::Mutex<HeavyOrtSession>>,
}

impl SessionSet {
    fn new(light: LightOrtSessions, heavy: HeavyOrtSession) -> Self {
        Self {
            light: Arc::new(light.into()),
            heavy: Arc::new(heavy.into()),
        }
    }

    fn get_light(&self) -> Arc<std::sync::Mutex<LightOrtSessions>> {
        self.light.clone()
    }

    fn get_heavy(&mut self) -> Arc<std::sync::Mutex<HeavyOrtSession>> {
        self.heavy.clone()
    }
}

struct LightOrtSessions {
    predict_duration: AssertSend<Session<'static>>,
    predict_intonation: AssertSend<Session<'static>>,
}

struct HeavyOrtSession {
    decode: AssertSend<Session<'static>>,
}

// FIXME: 以下のことをちゃんと確認した後、onnxruntime-rs側で`Session`が`Send`であると宣言する。
// https://github.com/VOICEVOX/voicevox_core/issues/307#issuecomment-1276184614

use self::assert_send::AssertSend;

mod assert_send {
    use onnxruntime::session::Session;

    pub(super) struct AssertSend<T>(T);

    impl<T> AssertSend<T> {
        pub(super) fn get_mut(&mut self) -> &mut T {
            &mut self.0
        }
    }

    impl From<Session<'static>> for AssertSend<Session<'static>> {
        fn from(session: Session<'static>) -> Self {
            Self(session)
        }
    }

    impl<T> AsRef<T> for AssertSend<T> {
        fn as_ref(&self) -> &T {
            &self.0
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
