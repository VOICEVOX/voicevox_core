use std::{fmt::Debug, vec};

use anyhow::{anyhow, ensure};
use duplicate::duplicate_item;
use ndarray::{Array, Dimension};
use ort::{
    CPUExecutionProvider, CUDAExecutionProvider, DirectMLExecutionProvider, ExecutionProvider as _,
    ExecutionProviderDispatch, GraphOptimizationLevel, IntoTensorElementType, TensorElementType,
    ValueType,
};

use crate::{devices::SupportedDevices, error::ErrorRepr};

use super::super::{
    DecryptModelError, InferenceRuntime, InferenceSessionOptions, InputScalarKind,
    OutputScalarKind, OutputTensor, ParamInfo, PushInputTensor,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Onnxruntime {}

impl InferenceRuntime for Onnxruntime {
    type Session = ort::Session;
    type RunContext<'a> = OnnxruntimeRunContext<'a>;

    fn supported_devices() -> crate::Result<SupportedDevices> {
        #![allow(unsafe_code)]

        // TODO: `InferenceRuntime::init`と`InitInferenceRuntimeError`を作る
        build_ort_env_once().unwrap();

        (|| {
            let cpu = CPUExecutionProvider::default().is_available()?;
            let cuda = CUDAExecutionProvider::default().is_available()?;
            let dml = DirectMLExecutionProvider::default().is_available()?;

            ensure!(cpu, "missing `CPUExecutionProvider`");

            Ok(SupportedDevices {
                cpu: true,
                cuda,
                dml,
            })
        })()
        .map_err(ErrorRepr::GetSupportedDevices)
        .map_err(Into::into)
    }

    fn new_session(
        model: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        options: InferenceSessionOptions,
    ) -> anyhow::Result<(
        Self::Session,
        Vec<ParamInfo<InputScalarKind>>,
        Vec<ParamInfo<OutputScalarKind>>,
    )> {
        // TODO: `InferenceRuntime::init`と`InitInferenceRuntimeError`を作る
        build_ort_env_once().unwrap();

        // TODO:
        // - with_intra_op_num_threads
        let builder = ort::Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level1)?
            .with_execution_providers([if options.use_gpu && cfg!(feature = "directml") {
                // TODO:
                // with_disable_mem_pattern
                // ExecutionMode::ORT_SEQUENTIAL
                ExecutionProviderDispatch::DirectML(Default::default())
            } else if options.use_gpu && cfg!(feature = "cuda") {
                ExecutionProviderDispatch::CUDA(Default::default())
            } else {
                ExecutionProviderDispatch::CPU(Default::default())
            }])?;

        let model = model()?;
        let sess = builder.with_model_from_memory(&{ model })?;

        let input_param_infos = sess
            .inputs
            .iter()
            .map(|info| {
                let ValueType::Tensor { ty, .. } = info.input_type else {
                    todo!()
                };

                let dt = match ty {
                    TensorElementType::Float32 => Ok(InputScalarKind::Float32),
                    TensorElementType::Uint8 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT8"),
                    TensorElementType::Int8 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT8"),
                    TensorElementType::Uint16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT16"),
                    TensorElementType::Int16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT16"),
                    TensorElementType::Int32 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT32"),
                    TensorElementType::Int64 => Ok(InputScalarKind::Int64),
                    TensorElementType::String => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_STRING"),
                    TensorElementType::Bfloat16 => todo!(),
                    TensorElementType::Float16 => todo!(),
                    TensorElementType::Float64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_DOUBLE"),
                    TensorElementType::Uint32 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT32"),
                    TensorElementType::Uint64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT64"),
                    TensorElementType::Bool => todo!(),
                }
                .map_err(|actual| {
                    anyhow!("unsupported input datatype `{actual}` for `{}`", info.name)
                })?;

                Ok(ParamInfo {
                    name: info.name.clone().into(),
                    dt,
                    ndim: info.input_type.tensor_dimensions().map(|d| d.len()),
                })
            })
            .collect::<anyhow::Result<_>>()?;

        let output_param_infos = sess
            .outputs
            .iter()
            .map(|info| {
                let ValueType::Tensor { ty, .. } = info.output_type else {
                    todo!()
                };

                let dt = match ty {
                    TensorElementType::Float32 => Ok(OutputScalarKind::Float32),
                    TensorElementType::Uint8 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT8"),
                    TensorElementType::Int8 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT8"),
                    TensorElementType::Uint16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT16"),
                    TensorElementType::Int16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT16"),
                    TensorElementType::Int32 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT32"),
                    TensorElementType::Int64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT64"),
                    TensorElementType::String => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_STRING"),
                    TensorElementType::Bfloat16 => todo!(),
                    TensorElementType::Float16 => todo!(),
                    TensorElementType::Float64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_DOUBLE"),
                    TensorElementType::Uint32 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT32"),
                    TensorElementType::Uint64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT64"),
                    TensorElementType::Bool => todo!(),
                }
                .map_err(|actual| {
                    anyhow!("unsupported output datatype `{actual}` for `{}`", info.name)
                })?;

                Ok(ParamInfo {
                    name: info.name.clone().into(),
                    dt,
                    ndim: info.output_type.tensor_dimensions().map(|d| d.len()),
                })
            })
            .collect::<anyhow::Result<_>>()?;

        Ok((sess, input_param_infos, output_param_infos))
    }

    fn run(
        OnnxruntimeRunContext { sess, inputs }: OnnxruntimeRunContext<'_>,
    ) -> anyhow::Result<Vec<OutputTensor>> {
        let outputs = sess.run(&*inputs)?;
        (0..outputs.len())
            .map(|i| {
                let output = &outputs[i];
                let dtype = output.dtype()?;

                if !matches!(
                    dtype,
                    ValueType::Tensor {
                        ty: TensorElementType::Float32,
                        ..
                    }
                ) {
                    todo!();
                }

                let tensor = output.extract_tensor::<f32>()?;
                Ok(OutputTensor::Float32(tensor.view().clone().into_owned()))
            })
            .collect()
    }
}

fn build_ort_env_once() -> ort::Result<()> {
    static ONCE: once_cell::sync::OnceCell<()> = once_cell::sync::OnceCell::new();

    // FIXME: ログレベルを絞る

    ONCE.get_or_try_init(|| ort::init().with_name(env!("CARGO_PKG_NAME")).commit())?;
    Ok(())
}

pub(crate) struct OnnxruntimeRunContext<'sess> {
    sess: &'sess mut ort::Session,
    inputs: Vec<ort::Value>,
}

impl OnnxruntimeRunContext<'_> {
    fn push_input(
        &mut self,
        input: Array<
            impl IntoTensorElementType + Debug + Clone + 'static,
            impl Dimension + 'static,
        >,
    ) {
        self.inputs
            .push(input.try_into().unwrap_or_else(|_| todo!()));
    }
}

impl<'sess> From<&'sess mut ort::Session> for OnnxruntimeRunContext<'sess> {
    fn from(sess: &'sess mut ort::Session) -> Self {
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
