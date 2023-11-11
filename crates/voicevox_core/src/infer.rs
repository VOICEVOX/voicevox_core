mod model_file;
pub(crate) mod runtimes;
pub(crate) mod signatures;

use std::{collections::HashMap, fmt::Debug, marker::PhantomData, sync::Arc};

use derive_new::new;
use easy_ext::ext;
use enum_map::{Enum, EnumMap};
use ndarray::{Array, ArrayD, Dimension, ShapeError};
use thiserror::Error;

use crate::{ErrorRepr, SupportedDevices};

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

pub(crate) struct InferenceSessionSet<G: InferenceGroup, R: InferenceRuntime>(
    EnumMap<G::Kind, Arc<std::sync::Mutex<R::Session>>>,
);

impl<G: InferenceGroup, R: InferenceRuntime> InferenceSessionSet<G, R> {
    pub(crate) fn new(
        model_bytes: &EnumMap<G::Kind, Vec<u8>>,
        mut options: impl FnMut(G::Kind) -> InferenceSessionOptions,
    ) -> anyhow::Result<Self> {
        let mut sessions = model_bytes
            .iter()
            .map(|(k, m)| {
                let sess = R::new_session(|| model_file::decrypt(m), options(k))?;
                Ok((k.into_usize(), std::sync::Mutex::new(sess).into()))
            })
            .collect::<anyhow::Result<HashMap<_, _>>>()?;

        Ok(Self(EnumMap::<G::Kind, _>::from_fn(|k| {
            sessions.remove(&k.into_usize()).expect("should exist")
        })))
    }
}

impl<G: InferenceGroup, R: InferenceRuntime> InferenceSessionSet<G, R> {
    pub(crate) fn get<I>(&self) -> InferenceSessionCell<R, I>
    where
        I: InferenceInputSignature,
        I::Signature: InferenceSignature<Group = G>,
    {
        InferenceSessionCell {
            inner: self.0[I::Signature::KIND].clone(),
            marker: PhantomData,
        }
    }
}

pub(crate) struct InferenceSessionCell<R: InferenceRuntime, I> {
    inner: Arc<std::sync::Mutex<R::Session>>,
    marker: PhantomData<fn(I)>,
}

impl<R: InferenceRuntime, I: InferenceInputSignature> InferenceSessionCell<R, I> {
    pub(crate) fn run(
        self,
        input: I,
    ) -> crate::Result<<I::Signature as InferenceSignature>::Output> {
        let inner = &mut self.inner.lock().unwrap();
        let ctx = input.make_run_context::<R>(inner);
        R::run(ctx)
            .and_then(TryInto::try_into)
            .map_err(|e| ErrorRepr::InferenceFailed(e).into())
    }
}

#[derive(new, Clone, Copy)]
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
