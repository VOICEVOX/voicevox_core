mod model_file;
pub(crate) mod runtimes;
pub(crate) mod signatures;

use std::{collections::HashMap, fmt::Debug, marker::PhantomData, sync::Arc};

use derive_new::new;
use easy_ext::ext;
use enum_map::{Enum, EnumMap};
use thiserror::Error;

use crate::{ErrorRepr, SupportedDevices};

pub(crate) trait InferenceRuntime: 'static {
    type Session: InferenceSession;
    type RunContext<'a>: RunContext<'a, Runtime = Self>;
    fn supported_devices() -> crate::Result<SupportedDevices>;
}

pub(crate) trait InferenceSession: Sized + Send + 'static {
    fn new(
        model: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        options: InferenceSessionOptions,
    ) -> anyhow::Result<Self>;
}

pub(crate) trait RunContext<'a>:
    From<&'a mut <Self::Runtime as InferenceRuntime>::Session>
{
    type Runtime: InferenceRuntime<RunContext<'a> = Self>;
}

#[ext(RunContextExt)]
impl<'a, T: RunContext<'a>> T {
    fn with_input<I>(mut self, tensor: I) -> Self
    where
        T::Runtime: SupportsInferenceInputTensor<I>,
    {
        T::Runtime::push_input(tensor, &mut self);
        self
    }
}

pub(crate) trait SupportsInferenceSignature<S: InferenceSignature>:
    SupportsInferenceInputSignature<S::Input> + SupportsInferenceOutput<S::Output>
{
}

impl<
        R: SupportsInferenceInputSignature<S::Input> + SupportsInferenceOutput<S::Output>,
        S: InferenceSignature,
    > SupportsInferenceSignature<S> for R
{
}

pub(crate) trait SupportsInferenceInputTensor<I>: InferenceRuntime {
    fn push_input(input: I, ctx: &mut Self::RunContext<'_>);
}

pub(crate) trait SupportsInferenceInputSignature<I: InferenceInputSignature>:
    InferenceRuntime
{
    fn make_run_context(sess: &mut Self::Session, input: I) -> Self::RunContext<'_>;
}

pub(crate) trait SupportsInferenceOutput<O: Send>: InferenceRuntime {
    fn run(ctx: Self::RunContext<'_>) -> anyhow::Result<O>;
}

pub(crate) trait InferenceGroup {
    type Kind: Copy + Enum;
}

pub(crate) trait InferenceSignature: Sized + Send + 'static {
    type Group: InferenceGroup;
    type Input: InferenceInputSignature<Signature = Self>;
    type Output: Send;
    const INFERENCE: <Self::Group as InferenceGroup>::Kind;
}

pub(crate) trait InferenceInputSignature: Send + 'static {
    type Signature: InferenceSignature<Input = Self>;
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
                let sess = R::Session::new(|| model_file::decrypt(m), options(k))?;
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
            inner: self.0[I::Signature::INFERENCE].clone(),
            marker: PhantomData,
        }
    }
}

pub(crate) struct InferenceSessionCell<R: InferenceRuntime, I> {
    inner: Arc<std::sync::Mutex<R::Session>>,
    marker: PhantomData<fn(I)>,
}

impl<
        R: SupportsInferenceInputSignature<I>
            + SupportsInferenceOutput<<I::Signature as InferenceSignature>::Output>,
        I: InferenceInputSignature,
    > InferenceSessionCell<R, I>
{
    pub(crate) fn run(
        self,
        input: I,
    ) -> crate::Result<<I::Signature as InferenceSignature>::Output> {
        let inner = &mut self.inner.lock().unwrap();
        let ctx = R::make_run_context(inner, input);
        R::run(ctx).map_err(|e| ErrorRepr::InferenceFailed(e).into())
    }
}

#[derive(new, Clone, Copy)]
pub(crate) struct InferenceSessionOptions {
    pub(crate) cpu_num_threads: u16,
    pub(crate) use_gpu: bool,
}

#[derive(Error, Debug)]
#[error("不正なモデルファイルです")]
pub(crate) struct DecryptModelError;
