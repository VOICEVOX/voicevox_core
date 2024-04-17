use std::{fmt::Debug, vec};

use anyhow::anyhow;
use duplicate::duplicate_item;
use ndarray::{Array, Dimension};
use once_cell::sync::Lazy;
use onnxruntime::{
    environment::Environment, GraphOptimizationLevel, LoggingLevel, TensorElementDataType,
    TypeToTensorElementDataType,
};

use crate::{devices::SupportedDevices, error::ErrorRepr};

use self::assert_send::AssertSend;

use super::super::{
    DecryptModelError, InferenceRuntime, InferenceSessionOptions, InputScalarKind,
    OutputScalarKind, OutputTensor, ParamInfo, PushInputTensor,
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
    ) -> anyhow::Result<(
        Self::Session,
        Vec<ParamInfo<InputScalarKind>>,
        Vec<ParamInfo<OutputScalarKind>>,
    )> {
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
        let sess = AssertSend::from(builder.with_model_from_memory(model)?);

        let input_param_infos = sess
            .inputs
            .iter()
            .map(|info| {
                let dt = match info.input_type {
                    TensorElementDataType::Float => Ok(InputScalarKind::Float32),
                    TensorElementDataType::Uint8 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT8"),
                    TensorElementDataType::Int8 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT8"),
                    TensorElementDataType::Uint16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT16"),
                    TensorElementDataType::Int16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT16"),
                    TensorElementDataType::Int32 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT32"),
                    TensorElementDataType::Int64 => Ok(InputScalarKind::Int64),
                    TensorElementDataType::String => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_STRING"),
                    TensorElementDataType::Double => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_DOUBLE"),
                    TensorElementDataType::Uint32 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT32"),
                    TensorElementDataType::Uint64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT64"),
                }
                .map_err(|actual| {
                    anyhow!("unsupported input datatype `{actual}` for `{}`", info.name)
                })?;

                Ok(ParamInfo {
                    name: info.name.clone().into(),
                    dt,
                    ndim: Some(info.dimensions.len()),
                })
            })
            .collect::<anyhow::Result<_>>()?;

        let output_param_infos = sess
            .outputs
            .iter()
            .map(|info| {
                let dt = match info.output_type {
                    TensorElementDataType::Float => Ok(OutputScalarKind::Float32),
                    TensorElementDataType::Uint8 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT8"),
                    TensorElementDataType::Int8 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT8"),
                    TensorElementDataType::Uint16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT16"),
                    TensorElementDataType::Int16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT16"),
                    TensorElementDataType::Int32 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT32"),
                    TensorElementDataType::Int64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT64"),
                    TensorElementDataType::String => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_STRING"),
                    TensorElementDataType::Double => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_DOUBLE"),
                    TensorElementDataType::Uint32 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT32"),
                    TensorElementDataType::Uint64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT64"),
                }
                .map_err(|actual| {
                    anyhow!("unsupported output datatype `{actual}` for `{}`", info.name)
                })?;

                Ok(ParamInfo {
                    name: info.name.clone().into(),
                    dt,
                    ndim: Some(info.dimensions.len()),
                })
            })
            .collect::<anyhow::Result<_>>()?;

        return Ok((sess, input_param_infos, output_param_infos));

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

impl OnnxruntimeRunContext<'_> {
    fn push_input(
        &mut self,
        input: Array<impl TypeToTensorElementDataType + Debug + 'static, impl Dimension + 'static>,
    ) {
        self.inputs
            .push(Box::new(onnxruntime::session::NdArray::new(input)));
    }
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

impl PushInputTensor for OnnxruntimeRunContext<'_> {
    #[duplicate_item(
        method           T;
        [ push_int64 ]   [ i64 ];
        [ push_float32 ] [ f32 ];
    )]
    fn method(&mut self, tensor: Array<T, impl Dimension + 'static>) {
        self.push_input(tensor);
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
    unsafe impl<T> Send for AssertSend<T> {}
}
