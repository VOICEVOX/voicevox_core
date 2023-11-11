use std::fmt::Debug;

use ndarray::{Array, Dimension};
use once_cell::sync::Lazy;
use onnxruntime::{
    environment::Environment, GraphOptimizationLevel, LoggingLevel, TensorElementDataType,
};

use self::assert_send::AssertSend;
use crate::{
    devices::SupportedDevices,
    error::ErrorRepr,
    infer::{
        DecryptModelError, InferenceRuntime, InferenceSessionOptions, InputScalar, OutputTensor,
        RunContext,
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

    fn new_session(
        model: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        options: InferenceSessionOptions,
    ) -> anyhow::Result<Self::Session> {
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
        let sess = builder.with_model_from_memory(model)?.into();
        return Ok(sess);

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

    fn push_input(
        input: Array<impl InputScalar, impl Dimension + 'static>,
        ctx: &mut Self::RunContext<'_>,
    ) {
        ctx.inputs
            .push(Box::new(onnxruntime::session::NdArray::new(input)));
    }

    fn run(
        OnnxruntimeRunContext { sess, mut inputs }: OnnxruntimeRunContext<'_>,
    ) -> anyhow::Result<Vec<OutputTensor>> {
        // FIXME: 現状では`f32`のみ対応。実行時にsessionからdatatypeが取れるので、別の型の対応も
        // おそらく可能ではあるが、それが必要になるよりもortクレートへの引越しが先になると思われる
        // のでこのままにする。

        if !sess
            .outputs
            .iter()
            .all(|info| matches!(info.output_type, TensorElementDataType::Float))
        {
            unimplemented!(
                "currently only `ONNX_TENSOR_ELEMENT_DATA_TYPE_FLOAT` is supported for output",
            );
        }

        let outputs = sess.run::<f32>(inputs.iter_mut().map(|t| &mut **t as &mut _).collect())?;

        Ok(outputs
            .iter()
            .map(|o| OutputTensor::Float32((*o).clone().into_owned()))
            .collect())
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
