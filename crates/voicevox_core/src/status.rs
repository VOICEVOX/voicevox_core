use super::*;
use once_cell::sync::Lazy;
use onnxruntime::{
    environment::Environment,
    session::{AnyArray, Session},
    GraphOptimizationLevel, LoggingLevel,
};
use serde::{Deserialize, Serialize};

cfg_if! {
    if #[cfg(not(feature="directml"))]{
        use onnxruntime::CudaProviderOptions;
    }
}
use std::collections::{BTreeMap, BTreeSet};

pub struct Status {
    models: StatusModels,
    light_session_options: SessionOptions, // 軽いモデルはこちらを使う
    heavy_session_options: SessionOptions, // 重いモデルはこちらを使う
    supported_styles: BTreeSet<usize>,
}

struct StatusModels {
    yukarin_s: BTreeMap<usize, Session<'static>>,
    yukarin_sa: BTreeMap<usize, Session<'static>>,
    decode: BTreeMap<usize, Session<'static>>,
}

#[derive(new, Getters)]
struct SessionOptions {
    cpu_num_threads: usize,
    use_gpu: bool,
}

struct Model {
    yukarin_s_model: &'static [u8],
    yukarin_sa_model: &'static [u8],
    decode_model: &'static [u8],
}

#[derive(Deserialize, Getters)]
struct Meta {
    styles: Vec<Style>,
}

#[derive(Deserialize, Getters)]
struct Style {
    id: u64,
}
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

#[derive(Getters, Debug, Serialize, Deserialize)]
pub struct SupportedDevices {
    cpu: bool,
    cuda: bool,
    dml: bool,
}

impl SupportedDevices {
    pub fn get_supported_devices() -> Result<Self> {
        let mut cuda_support = false;
        let mut dml_support = false;
        for provider in onnxruntime::session::get_available_providers()
            .map_err(|e| Error::GetSupportedDevices(e.into()))?
            .iter()
        {
            match provider.as_str() {
                "CUDAExecutionProvider" => cuda_support = true,
                "DmlExecutionProvider" => dml_support = true,
                _ => {}
            }
        }

        Ok(SupportedDevices {
            cpu: true,
            cuda: cuda_support,
            dml: dml_support,
        })
    }
}

#[allow(unsafe_code)]
unsafe impl Send for Status {}
#[allow(unsafe_code)]
unsafe impl Sync for Status {}

impl Status {
    const MODELS: &'static [Model] = &include!("include_models.rs");

    pub const METAS_STR: &'static str =
        include_str!(concat!(env!("CARGO_WORKSPACE_DIR"), "/model/metas.json"));

    pub const MODELS_COUNT: usize = Self::MODELS.len();

    pub fn new(use_gpu: bool, cpu_num_threads: usize) -> Self {
        Self {
            models: StatusModels {
                yukarin_s: BTreeMap::new(),
                yukarin_sa: BTreeMap::new(),
                decode: BTreeMap::new(),
            },
            light_session_options: SessionOptions::new(cpu_num_threads, false),
            heavy_session_options: SessionOptions::new(cpu_num_threads, use_gpu),
            supported_styles: BTreeSet::default(),
        }
    }

    pub fn load_metas(&mut self) -> Result<()> {
        let metas: Vec<Meta> =
            serde_json::from_str(Self::METAS_STR).map_err(|e| Error::LoadMetas(e.into()))?;

        for meta in metas.iter() {
            for style in meta.styles().iter() {
                self.supported_styles.insert(*style.id() as usize);
            }
        }

        Ok(())
    }

    pub fn load_model(&mut self, model_index: usize) -> Result<()> {
        if model_index < Self::MODELS.len() {
            let model = &Self::MODELS[model_index];
            let yukarin_s_session = self
                .new_session(model.yukarin_s_model, &self.light_session_options)
                .map_err(Error::LoadModel)?;
            let yukarin_sa_session = self
                .new_session(model.yukarin_sa_model, &self.light_session_options)
                .map_err(Error::LoadModel)?;
            let decode_model = self
                .new_session(model.decode_model, &self.heavy_session_options)
                .map_err(Error::LoadModel)?;

            self.models.yukarin_s.insert(model_index, yukarin_s_session);
            self.models
                .yukarin_sa
                .insert(model_index, yukarin_sa_session);

            self.models.decode.insert(model_index, decode_model);

            Ok(())
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }

    pub fn is_model_loaded(&self, model_index: usize) -> bool {
        self.models.yukarin_sa.contains_key(&model_index)
            && self.models.yukarin_s.contains_key(&model_index)
            && self.models.decode.contains_key(&model_index)
    }

    fn new_session<B: AsRef<[u8]>>(
        &self,
        model_bytes: B,
        session_options: &SessionOptions,
    ) -> std::result::Result<Session<'static>, SourceError> {
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

        Ok(session_builder.with_model_from_memory(model_bytes)?)
    }

    pub fn validate_speaker_id(&self, speaker_id: usize) -> bool {
        self.supported_styles.contains(&speaker_id)
    }

    pub fn yukarin_s_session_run(
        &mut self,
        model_index: usize,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<f32>> {
        if let Some(model) = self.models.yukarin_s.get_mut(&model_index) {
            if let Ok(output_tensors) = model.run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }

    pub fn yukarin_sa_session_run(
        &mut self,
        model_index: usize,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<f32>> {
        if let Some(model) = self.models.yukarin_sa.get_mut(&model_index) {
            if let Ok(output_tensors) = model.run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }

    pub fn decode_session_run(
        &mut self,
        model_index: usize,
        inputs: Vec<&mut dyn AnyArray>,
    ) -> Result<Vec<f32>> {
        if let Some(model) = self.models.decode.get_mut(&model_index) {
            if let Ok(output_tensors) = model.run(inputs) {
                Ok(output_tensors[0].as_slice().unwrap().to_owned())
            } else {
                Err(Error::InferenceFailed)
            }
        } else {
            Err(Error::InvalidModelIndex { model_index })
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[rstest]
    #[case(true, 0)]
    #[case(true, 1)]
    #[case(true, 8)]
    #[case(false, 2)]
    #[case(false, 4)]
    #[case(false, 8)]
    #[case(false, 0)]
    fn status_new_works(#[case] use_gpu: bool, #[case] cpu_num_threads: usize) {
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
        assert!(status.models.yukarin_s.is_empty());
        assert!(status.models.yukarin_sa.is_empty());
        assert!(status.models.decode.is_empty());
        assert!(status.supported_styles.is_empty());
    }

    #[rstest]
    fn status_load_metas_works() {
        let mut status = Status::new(true, 0);
        let result = status.load_metas();
        assert_eq!(Ok(()), result);
        let mut expected = BTreeSet::new();
        expected.insert(0);
        expected.insert(1);
        assert_eq!(expected, status.supported_styles);
    }

    #[rstest]
    fn supported_devices_get_supported_devices_works() {
        let result = SupportedDevices::get_supported_devices();
        // 環境によって結果が変わるので、関数呼び出しが成功するかどうかの確認のみ行う
        assert!(result.is_ok(), "{:?}", result);
    }

    #[rstest]
    fn status_load_model_works() {
        let mut status = Status::new(false, 0);
        let result = status.load_model(0);
        assert_eq!(Ok(()), result);
        assert_eq!(1, status.models.yukarin_s.len());
        assert_eq!(1, status.models.yukarin_sa.len());
        assert_eq!(1, status.models.decode.len());
    }

    #[rstest]
    fn status_is_model_loaded_works() {
        let mut status = Status::new(false, 0);
        let model_index = 0;
        assert!(
            !status.is_model_loaded(model_index),
            "model should  not be loaded"
        );
        let result = status.load_model(model_index);
        assert_eq!(Ok(()), result);
        assert!(
            status.is_model_loaded(model_index),
            "model should be loaded"
        );
    }
}
