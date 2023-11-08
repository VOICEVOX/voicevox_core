use std::fmt::Debug;

use ndarray::{Array, Dimension};
use once_cell::sync::Lazy;
use onnxruntime::{
    environment::Environment, GraphOptimizationLevel, LoggingLevel, TypeToTensorElementDataType,
};

use self::assert_send::AssertSend;
use crate::{
    devices::SupportedDevices,
    error::ErrorRepr,
    infer::{
        DecryptModelError, InferenceRuntime, InferenceSession, InferenceSessionOptions, RunContext,
        SupportsInferenceInputTensor, SupportsInferenceOutput,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Onnxruntime {}

impl InferenceRuntime for Onnxruntime {
    type Session = AssertSend<onnxruntime::session::Session<'static>>;
    type RunContext<'a> = OnnxruntimeRunContext<'a>;

    fn supported_devices() -> crate::Result<SupportedDevices> {
        let mut cuda_support = false;
        let mut dml_support = false;
        for provider in onnxruntime::session::get_available_providers()
            .map_err(Into::into)
            .map_err(ErrorRepr::GetSupportedDevices)?
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

impl InferenceSession for AssertSend<onnxruntime::session::Session<'static>> {
    fn new(
        model: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        options: InferenceSessionOptions,
    ) -> anyhow::Result<Self> {
        let mut builder = ENVIRONMENT
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::Basic)?
            .with_intra_op_num_threads(options.cpu_num_threads.into())?
            .with_inter_op_num_threads(options.cpu_num_threads.into())?;

        if options.use_gpu {
            #[cfg(feature = "directml")]
            {
                use onnxruntime::ExecutionMode;

                builder = builder
                    .with_disable_mem_pattern()?
                    .with_execution_mode(ExecutionMode::ORT_SEQUENTIAL)?
                    .with_append_execution_provider_directml(0)?;
            }

            #[cfg(not(feature = "directml"))]
            {
                builder = builder.with_append_execution_provider_cuda(Default::default())?;
            }
        }

        let model = model()?;
        let this = builder.with_model_from_memory(model)?.into();
        return Ok(this);

        static ENVIRONMENT: Lazy<Environment> = Lazy::new(|| {
            Environment::builder()
                .with_name(env!("CARGO_PKG_NAME"))
                .with_log_level(LOGGING_LEVEL)
                .build()
                .unwrap()
        });

        const LOGGING_LEVEL: LoggingLevel = if cfg!(debug_assertions) {
            LoggingLevel::Verbose
        } else {
            LoggingLevel::Warning
        };
    }
}

pub(crate) struct OnnxruntimeRunContext<'sess> {
    sess: &'sess mut AssertSend<onnxruntime::session::Session<'static>>,
    inputs: Vec<Box<dyn onnxruntime::session::AnyArray>>,
}

impl<'sess> From<&'sess mut AssertSend<onnxruntime::session::Session<'static>>>
    for OnnxruntimeRunContext<'sess>
{
    fn from(sess: &'sess mut AssertSend<onnxruntime::session::Session<'static>>) -> Self {
        Self {
            sess,
            inputs: vec![],
        }
    }
}

impl<'sess> RunContext<'sess> for OnnxruntimeRunContext<'sess> {
    type Runtime = Onnxruntime;
}

impl<A: TypeToTensorElementDataType + Debug + 'static, D: Dimension + 'static>
    SupportsInferenceInputTensor<Array<A, D>> for Onnxruntime
{
    fn input(tensor: Array<A, D>, ctx: &mut Self::RunContext<'_>) {
        ctx.inputs
            .push(Box::new(onnxruntime::session::NdArray::new(tensor)));
    }
}

impl SupportsInferenceOutput<(Vec<f32>,)> for Onnxruntime {
    fn run(
        OnnxruntimeRunContext { sess, mut inputs }: OnnxruntimeRunContext<'_>,
    ) -> anyhow::Result<(Vec<f32>,)> {
        let outputs = sess.run(inputs.iter_mut().map(|t| &mut **t as &mut _).collect())?;

        // FIXME: 2個以上の出力や二次元以上の出力をちゃんとしたやりかたで弾く
        Ok((outputs[0].as_slice().unwrap().to_owned(),))
    }
}

// FIXME: 以下のことをちゃんと確認した後、onnxruntime-rs側で`Session`が`Send`であると宣言する。
// https://github.com/VOICEVOX/voicevox_core/issues/307#issuecomment-1276184614
mod assert_send {
    use std::ops::{Deref, DerefMut};

    pub(crate) struct AssertSend<T>(T);

    impl From<onnxruntime::session::Session<'static>>
        for AssertSend<onnxruntime::session::Session<'static>>
    {
        fn from(session: onnxruntime::session::Session<'static>) -> Self {
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
