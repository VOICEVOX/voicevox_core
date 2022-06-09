use super::*;
use once_cell::sync::Lazy;
use onnxruntime::{
    environment::Environment, session::Session, GraphOptimizationLevel, LoggingLevel,
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
    session_options: SessionOptions,
    supported_styles: BTreeSet<u64>,
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

unsafe impl Send for Status {}
unsafe impl Sync for Status {}

impl Status {
    const YUKARIN_S_MODEL: &'static [u8] = include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"),
        "/model/yukarin_s.onnx"
    ));
    const YUKARIN_SA_MODEL: &'static [u8] = include_bytes!(concat!(
        env!("CARGO_WORKSPACE_DIR"),
        "/model/yukarin_sa.onnx"
    ));

    const DECODE_MODEL: &'static [u8] =
        include_bytes!(concat!(env!("CARGO_WORKSPACE_DIR"), "/model/decode.onnx"));

    const MODELS: [Model; 1] = [Model {
        yukarin_s_model: Self::YUKARIN_S_MODEL,
        yukarin_sa_model: Self::YUKARIN_SA_MODEL,
        decode_model: Self::DECODE_MODEL,
    }];

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
            session_options: SessionOptions::new(cpu_num_threads, use_gpu),
            supported_styles: BTreeSet::default(),
        }
    }

    pub fn load_metas(&mut self) -> Result<()> {
        let metas: Vec<Meta> =
            serde_json::from_str(Self::METAS_STR).map_err(|e| Error::LoadMetas(e.into()))?;

        for meta in metas.iter() {
            for style in meta.styles().iter() {
                self.supported_styles.insert(*style.id());
            }
        }

        Ok(())
    }

    pub fn load_model(&mut self, model_index: usize) -> Result<()> {
        if model_index < Self::MODELS.len() {
            let model = &Self::MODELS[model_index];
            let yukarin_s_session = self
                .new_session(model.yukarin_s_model)
                .map_err(Error::LoadModel)?;
            let yukarin_sa_session = self
                .new_session(model.yukarin_sa_model)
                .map_err(Error::LoadModel)?;
            let decode_model = self
                .new_session(model.decode_model)
                .map_err(Error::LoadModel)?;

            self.models.yukarin_s.insert(model_index, yukarin_s_session);
            self.models
                .yukarin_sa
                .insert(model_index, yukarin_sa_session);

            self.models.decode.insert(model_index, decode_model);

            Ok(())
        } else {
            Err(Error::InvalidModelIndex(model_index))
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
    ) -> std::result::Result<Session<'static>, anyhow::Error> {
        let session_builder = ENVIRONMENT
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::Basic)?
            .with_intra_op_num_threads(*self.session_options.cpu_num_threads() as i32)?
            .with_inter_op_num_threads(*self.session_options.cpu_num_threads() as i32)?;

        let session_builder = if *self.session_options.use_gpu() {
            cfg_if! {
                if #[cfg(feature = "directml")]{
                    session_builder
                        .with_disable_mem_pattern()?
                        .with_execution_mode(onnxruntime::ExecutionMode::ORT_SEQUENTIAL)?
                } else {
                    let options = CudaProviderOptions::default();
                    session_builder
                        .with_disable_mem_pattern()?
                        .with_append_execution_provider_cuda(options)?
                }
            }
        } else {
            session_builder
        };

        Ok(session_builder.with_model_from_memory(model_bytes)?)
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
        assert_eq!(use_gpu, status.session_options.use_gpu);
        assert_eq!(cpu_num_threads, status.session_options.cpu_num_threads);
        assert!(status.models.yukarin_s.is_empty());
        assert!(status.models.yukarin_sa.is_empty());
        assert!(status.models.decode.is_empty());
        assert!(status.supported_styles.is_empty());
    }

    #[rstest]
    fn status_load_metas_works() {
        let mut status = Status::new(true, 0);
        let result = status.load_metas();
        assert!(result.is_ok(), "{:?}", result);
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
        assert!(result.is_ok(), "{:?}", result);
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
        assert!(result.is_ok(), "{:?}", result);
        assert!(
            status.is_model_loaded(model_index),
            "model should be loaded"
        );
    }
}
