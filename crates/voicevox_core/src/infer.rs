mod model_file;
pub(crate) mod runtimes;
pub(crate) mod signatures;

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use derive_new::new;
use enum_map::{Enum, EnumMap};
use ndarray::{Array, Dimension, LinalgScalar};
use thiserror::Error;

use crate::{ErrorRepr, SupportedDevices};

pub(crate) trait InferenceRuntime: 'static {
    type Session: Session;
    type RunBuilder<'a>: RunBuilder<'a, Session = Self::Session>;
    fn supported_devices() -> crate::Result<SupportedDevices>;
}

pub(crate) trait Session: Sized + Send + 'static {
    fn new(
        model: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        options: SessionOptions,
    ) -> anyhow::Result<Self>;
}

pub(crate) trait RunBuilder<'a>: From<&'a mut Self::Session> {
    type Session: Session;
    fn input(&mut self, tensor: Array<impl InputScalar, impl Dimension + 'static>) -> &mut Self;
}

pub(crate) trait InputScalar: LinalgScalar + Debug + sealed::OnnxruntimeInputScalar {}

impl InputScalar for i64 {}
impl InputScalar for f32 {}

pub(crate) trait Signature: Sized + Send + 'static {
    type Kind: Enum;
    type Output;
    const KIND: Self::Kind;
    fn input<'a, 'b>(self, ctx: &'a mut impl RunBuilder<'b>);
}

pub(crate) trait Output<R: InferenceRuntime>: Sized + Send {
    fn run(ctx: R::RunBuilder<'_>) -> anyhow::Result<Self>;
}

pub(crate) struct SessionSet<K: Enum, R: InferenceRuntime>(
    EnumMap<K, Arc<std::sync::Mutex<R::Session>>>,
);

impl<K: Enum, R: InferenceRuntime> SessionSet<K, R> {
    pub(crate) fn new(
        model_bytes: &EnumMap<K, Vec<u8>>,
        mut options: impl FnMut(K) -> SessionOptions,
    ) -> anyhow::Result<Self> {
        let mut sessions = model_bytes
            .iter()
            .map(|(k, m)| {
                let sess = R::Session::new(|| model_file::decrypt(m), options(k))?;
                Ok(Some(Arc::new(std::sync::Mutex::new(sess))))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self(EnumMap::<K, _>::from_fn(|k| {
            sessions[k.into_usize()].take().expect("should exist")
        })))
    }
}

impl<K: Enum, R: InferenceRuntime> SessionSet<K, R> {
    pub(crate) fn get<S: Signature<Kind = K>>(&self) -> SessionCell<R, S> {
        SessionCell {
            inner: self.0[S::KIND].clone(),
            marker: PhantomData,
        }
    }
}

pub(crate) struct SessionCell<R: InferenceRuntime, S> {
    inner: Arc<std::sync::Mutex<R::Session>>,
    marker: PhantomData<fn(S)>,
}

impl<R: InferenceRuntime, S: Signature> SessionCell<R, S> {
    pub(crate) fn run(self, input: S) -> crate::Result<S::Output>
    where
        S::Output: Output<R>,
    {
        let mut inner = self.inner.lock().unwrap();
        let mut ctx = R::RunBuilder::from(&mut inner);
        input.input(&mut ctx);
        S::Output::run(ctx).map_err(|e| ErrorRepr::InferenceFailed(e).into())
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
