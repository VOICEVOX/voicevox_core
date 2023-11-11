mod model_file;
pub(crate) mod runtimes;
pub(crate) mod signatures;
pub(crate) mod status;

use std::fmt::Debug;

use derive_new::new;
use easy_ext::ext;
use enum_map::Enum;
use ndarray::{Array, ArrayD, Dimension, ShapeError};
use thiserror::Error;

use crate::SupportedDevices;

pub(crate) trait InferenceRuntime: 'static {
    type Session: Sized + Send + 'static;
    type RunContext<'a>: RunContext<'a, Runtime = Self>;

    fn supported_devices() -> crate::Result<SupportedDevices>;

    fn new_session(
        model: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        options: InferenceSessionOptions,
    ) -> anyhow::Result<Self::Session>;

    fn push_input(
        input: Array<impl InputScalar, impl Dimension + 'static>,
        ctx: &mut Self::RunContext<'_>,
    );

    fn run(ctx: Self::RunContext<'_>) -> anyhow::Result<Vec<OutputTensor>>;
}

pub(crate) trait RunContext<'a>:
    From<&'a mut <Self::Runtime as InferenceRuntime>::Session>
{
    type Runtime: InferenceRuntime<RunContext<'a> = Self>;
}

#[ext(RunContextExt)]
impl<'a, T: RunContext<'a>> T {
    fn with_input(mut self, tensor: Array<impl InputScalar, impl Dimension + 'static>) -> Self {
        T::Runtime::push_input(tensor, &mut self);
        self
    }
}

pub(crate) trait InferenceGroup {
    type Kind: Copy + Enum;
}

pub(crate) trait InferenceSignature: Sized + Send + 'static {
    type Group: InferenceGroup;
    type Input: InferenceInputSignature<Signature = Self>;
    type Output: TryFrom<Vec<OutputTensor>, Error = anyhow::Error> + Send;
    const KIND: <Self::Group as InferenceGroup>::Kind;
}

pub(crate) trait InferenceInputSignature: Send + 'static {
    type Signature: InferenceSignature<Input = Self>;
    fn make_run_context<R: InferenceRuntime>(self, sess: &mut R::Session) -> R::RunContext<'_>;
}

pub(crate) trait InputScalar: sealed::InputScalar + Debug + 'static {}

impl InputScalar for i64 {}
impl InputScalar for f32 {}

pub(crate) trait OutputScalar: Sized {
    fn extract(tensor: OutputTensor) -> std::result::Result<ArrayD<Self>, ExtractError>;
}

impl OutputScalar for f32 {
    fn extract(tensor: OutputTensor) -> std::result::Result<ArrayD<Self>, ExtractError> {
        match tensor {
            OutputTensor::Float32(tensor) => Ok(tensor),
        }
    }
}

pub(crate) enum OutputTensor {
    Float32(ArrayD<f32>),
}

impl<A: OutputScalar, D: Dimension> TryFrom<OutputTensor> for Array<A, D> {
    type Error = ExtractError;

    fn try_from(tensor: OutputTensor) -> Result<Self, Self::Error> {
        let this = A::extract(tensor)?.into_dimensionality()?;
        Ok(this)
    }
}

#[derive(new, Clone, Copy, PartialEq, Debug)]
pub(crate) struct InferenceSessionOptions {
    pub(crate) cpu_num_threads: u16,
    pub(crate) use_gpu: bool,
}

#[derive(Error, Debug)]
pub(crate) enum ExtractError {
    #[error(transparent)]
    Shape(#[from] ShapeError),
}

#[derive(Error, Debug)]
#[error("不正なモデルファイルです")]
pub(crate) struct DecryptModelError;

mod sealed {
    pub(crate) trait InputScalar: OnnxruntimeInputScalar {}

    impl InputScalar for i64 {}
    impl InputScalar for f32 {}

    pub(crate) trait OnnxruntimeInputScalar:
        onnxruntime::TypeToTensorElementDataType
    {
    }

    impl<T: onnxruntime::TypeToTensorElementDataType> OnnxruntimeInputScalar for T {}
}
