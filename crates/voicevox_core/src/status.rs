use super::*;
use once_cell::sync::Lazy;
use onnxruntime::{
    environment::Environment,
    session::{AnyArray, Session},
    GraphOptimizationLevel, LoggingLevel,
};
use std::sync::Mutex;
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
    models: StatusModels,
    merged_metas: VoiceModelMeta,
    light_session_options: SessionOptions, // 軽いモデルはこちらを使う
    heavy_session_options: SessionOptions, // 重いモデルはこちらを使う
    pub id_relations: BTreeMap<StyleId, (VoiceModelId, ModelInnerId)>, // FIXME: pubはやめたい
}

struct StatusModels {
    metas: BTreeMap<VoiceModelId, VoiceModelMeta>,
    predict_duration: BTreeMap<VoiceModelId, Mutex<Session<'static>>>,
    predict_intonation: BTreeMap<VoiceModelId, Mutex<Session<'static>>>,
    decode: BTreeMap<VoiceModelId, Mutex<Session<'static>>>,
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

#[allow(unsafe_code)]
unsafe impl Send for Status {}

#[allow(unsafe_code)]
unsafe impl Sync for Status {}

impl Status {
    pub fn new(use_gpu: bool, cpu_num_threads: u16) -> Self {
        Self {
            models: StatusModels {
                metas: BTreeMap::new(),
                predict_duration: BTreeMap::new(),
                predict_intonation: BTreeMap::new(),
                decode: BTreeMap::new(),
            },
            merged_metas: VoiceModelMeta::default(),
            light_session_options: SessionOptions::new(cpu_num_threads, false),
            heavy_session_options: SessionOptions::new(cpu_num_threads, use_gpu),
            id_relations: BTreeMap::default(),
        }
    }

    pub async fn load_model(&mut self, model: &VoiceModel) -> Result<()> {
        for speaker in model.metas().iter() {
            for style in speaker.styles().iter() {
                if self.id_relations.contains_key(style.id()) {
                    Err(Error::AlreadyLoadedModel {
                        path: model.path().clone(),
                    })?;
                }
            }
        }
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
        self.models
            .metas
            .insert(model.id().clone(), model.metas().clone());

        for speaker in model.metas().iter() {
            for style in speaker.styles().iter() {
                self.id_relations.insert(
                    *style.id(),
                    (model.id().clone(), model.model_inner_id_for(*style.id())),
                );
            }
        }
        self.set_metas();

        self.models
            .predict_duration
            .insert(model.id().clone(), Mutex::new(predict_duration_session));
        self.models
            .predict_intonation
            .insert(model.id().clone(), Mutex::new(predict_intonation_session));

        self.models
            .decode
            .insert(model.id().clone(), Mutex::new(decode_model));

        Ok(())
    }

    pub fn unload_model(&mut self, voice_model_id: &VoiceModelId) -> Result<()> {
        if self.is_loaded_model(voice_model_id) {
            self.models.predict_intonation.remove(voice_model_id);
            self.models.predict_duration.remove(voice_model_id);
            self.models.decode.remove(voice_model_id);

            let remove_style_ids = self
                .id_relations
                .iter()
                .filter(|&(_, (loaded_model_id, _))| loaded_model_id == voice_model_id)
                .map(|(&style_id, _)| style_id)
                .collect::<Vec<_>>();

            for style_id in remove_style_ids.iter() {
                self.id_relations.remove(style_id);
            }
            self.set_metas();
            Ok(())
        } else {
            Err(Error::UnloadedModel {
                model_id: voice_model_id.clone(),
            })
        }
    }

    fn set_metas(&mut self) {
        let mut meta = VoiceModelMeta::default();
        for m in self.models.metas.values() {
            meta.extend_from_slice(m);
        }
        self.merged_metas = meta;
    }

    pub fn metas(&self) -> &VoiceModelMeta {
        &self.merged_metas
    }

    pub fn is_loaded_model(&self, voice_model_id: &VoiceModelId) -> bool {
        self.models.predict_duration.contains_key(voice_model_id)
            && self.models.predict_intonation.contains_key(voice_model_id)
            && self.models.decode.contains_key(voice_model_id)
    }

    pub fn is_loaded_model_by_style_id(&self, style_id: StyleId) -> bool {
        self.id_relations.contains_key(&style_id)
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
        self.id_relations.contains_key(&style_id)
    }

    pub fn predict_duration_session_run(
        &self,
        model_id: &VoiceModelId,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<f32>> {
        if let Some(model) = self.models.predict_duration.get(model_id) {
            if let Ok(output_tensors) = model.lock().unwrap().run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelId {
                model_id: model_id.clone(),
            })
        }
    }

    pub fn predict_intonation_session_run(
        &self,
        model_id: &VoiceModelId,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<f32>> {
        if let Some(model) = self.models.predict_intonation.get(model_id) {
            if let Ok(output_tensors) = model.lock().unwrap().run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelId {
                model_id: model_id.clone(),
            })
        }
    }

    pub fn decode_session_run(
        &self,
        model_id: &VoiceModelId,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<f32>> {
        if let Some(model) = self.models.decode.get(model_id) {
            if let Ok(output_tensors) = model.lock().unwrap().run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelId {
                model_id: model_id.clone(),
            })
        }
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
        assert!(status.models.predict_duration.is_empty());
        assert!(status.models.predict_intonation.is_empty());
        assert!(status.models.decode.is_empty());
        assert!(status.id_relations.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn status_load_model_works() {
        let mut status = Status::new(false, 0);
        let result = status.load_model(&open_default_vvm_file().await).await;
        assert_debug_fmt_eq!(Ok(()), result);
        assert_eq!(1, status.models.predict_duration.len());
        assert_eq!(1, status.models.predict_intonation.len());
        assert_eq!(1, status.models.decode.len());
    }

    #[rstest]
    #[tokio::test]
    async fn status_is_model_loaded_works() {
        let mut status = Status::new(false, 0);
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
