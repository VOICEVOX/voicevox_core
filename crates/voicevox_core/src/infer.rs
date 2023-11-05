pub(crate) mod runtimes;
pub(crate) mod signatures;

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use derive_new::new;
use ndarray::{Array, Dimension, LinalgScalar};
use thiserror::Error;

pub(crate) trait InferenceRuntime: Copy {
    type Session: Session;
    type RunBuilder<'a>: RunBuilder<'a, Runtime = Self>;
}

pub(crate) trait Session: Sized + 'static {
    fn new(
        model: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        options: SessionOptions,
    ) -> anyhow::Result<Self>;
}

pub(crate) trait RunBuilder<'a>:
    From<&'a mut <Self::Runtime as InferenceRuntime>::Session>
{
    type Runtime: InferenceRuntime;
    fn input(&mut self, tensor: Array<impl InputScalar, impl Dimension + 'static>) -> &mut Self;
}

pub(crate) trait InputScalar: LinalgScalar + Debug + sealed::OnnxruntimeInputScalar {}

impl InputScalar for i64 {}
impl InputScalar for f32 {}

pub(crate) trait Signature: Sized + Send + Sync + 'static {
    type SessionSet<R: InferenceRuntime>;
    type Output;
    fn get_session<R: InferenceRuntime>(
        session_set: &Self::SessionSet<R>,
    ) -> &Arc<std::sync::Mutex<TypedSession<R, Self>>>;
    fn input<'a, 'b>(self, ctx: &'a mut impl RunBuilder<'b>);
}

pub(crate) trait Output<R: InferenceRuntime>: Sized + Send {
    fn run(ctx: R::RunBuilder<'_>) -> anyhow::Result<Self>;
}

pub(crate) struct TypedSession<R: InferenceRuntime, I> {
    inner: R::Session,
    marker: PhantomData<fn(I)>,
}

impl<R: InferenceRuntime, S: Signature> TypedSession<R, S> {
    pub(crate) fn new(
        model: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        options: SessionOptions,
    ) -> anyhow::Result<Self> {
        let inner = R::Session::new(model, options)?;
        Ok(Self {
            inner,
            marker: PhantomData,
        })
    }

    pub(crate) fn run(&mut self, sig: S) -> anyhow::Result<S::Output>
    where
        S::Output: Output<R>,
    {
        let mut ctx = R::RunBuilder::from(&mut self.inner);
        sig.input(&mut ctx);
        S::Output::run(ctx)
    }
}

#[derive(new, Clone, Copy)]
pub(crate) struct SessionOptions {
    pub(crate) cpu_num_threads: u16,
    pub(crate) use_gpu: bool,
}

#[derive(Error, Debug)]
#[error("不正なモデルファイルです")]
pub(crate) struct DecryptModelError;

mod sealed {
    pub(crate) trait OnnxruntimeInputScalar:
        onnxruntime::TypeToTensorElementDataType
    {
    }

    impl OnnxruntimeInputScalar for i64 {}
    impl OnnxruntimeInputScalar for f32 {}
}
