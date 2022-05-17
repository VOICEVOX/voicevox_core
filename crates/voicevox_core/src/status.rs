use super::*;
use once_cell::sync::Lazy;
use onnxruntime::{
    environment::Environment, session::Session, CudaProviderOptions, GraphOptimizationLevel,
    LoggingLevel,
};
use std::collections::BTreeMap;
use std::sync::Mutex;

pub struct Status {
    models: Mutex<StatusModels>,
    session_options: SessionOptions,
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

#[derive(Getters)]
pub struct SupportedDevices {
    // supported_devices関数を実装したらこのattributeをはずす
    #[allow(dead_code)]
    cpu: bool,
    cuda: bool,
    // supported_devices関数を実装したらこのattributeをはずす
    #[allow(dead_code)]
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
    pub const MODELS_COUNT: usize = Self::MODELS.len();

    pub fn new(use_gpu: bool, cpu_num_threads: usize) -> Self {
        Self {
            models: Mutex::new(StatusModels {
                yukarin_s: BTreeMap::new(),
                yukarin_sa: BTreeMap::new(),
                decode: BTreeMap::new(),
            }),
            session_options: SessionOptions::new(cpu_num_threads, use_gpu),
        }
    }

    pub fn load_model(&mut self, model_index: usize) -> Result<()> {
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

        let mut models = self.models.lock().unwrap();
        models.yukarin_s.insert(model_index, yukarin_s_session);
        models.yukarin_sa.insert(model_index, yukarin_sa_session);

        models.decode.insert(model_index, decode_model);

        Ok(())
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
