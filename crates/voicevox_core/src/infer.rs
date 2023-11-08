mod model_file;
pub(crate) mod runtimes;
pub(crate) mod signatures;

use std::{collections::HashMap, fmt::Debug, marker::PhantomData, sync::Arc};

use derive_new::new;
use enum_map::{Enum, EnumMap};
use thiserror::Error;

use crate::{ErrorRepr, SupportedDevices};

pub(crate) trait InferenceRuntime: 'static {
    type Session: InferenceSession;
    type RunContext<'a>: RunContext<'a, Session = Self::Session>;
    fn supported_devices() -> crate::Result<SupportedDevices>;
}

pub(crate) trait InferenceSession: Sized + Send + 'static {
    fn new(
        model: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        options: InferenceSessionOptions,
    ) -> anyhow::Result<Self>;
}

pub(crate) trait RunContext<'a>: From<&'a mut Self::Session> {
    type Session: InferenceSession;
}

pub(crate) trait SupportsInferenceSignature<S: InferenceSignature>:
    SupportsInferenceInputTensors<S::Input> + SupportsInferenceOutput<S::Output>
{
}

impl<
        R: SupportsInferenceInputTensors<S::Input> + SupportsInferenceOutput<S::Output>,
        S: InferenceSignature,
    > SupportsInferenceSignature<S> for R
{
}

pub(crate) trait SupportsInferenceInputTensor<I>: InferenceRuntime {
    fn input(ctx: &mut Self::RunContext<'_>, tensor: I);
}

pub(crate) trait SupportsInferenceInputTensors<I: InferenceInput>: InferenceRuntime {
    fn input(ctx: &mut Self::RunContext<'_>, tensors: I);
}

pub(crate) trait SupportsInferenceOutput<O: Send>: InferenceRuntime {
    fn run(ctx: Self::RunContext<'_>) -> anyhow::Result<O>;
}

pub(crate) trait InferenceSignature: Sized + Send + 'static {
    type Kind: Enum + Copy;
    type Input: InferenceInput;
    type Output: Send;
    const KIND: Self::Kind;
}

pub(crate) trait InferenceInput: Send + 'static {
    type Signature: InferenceSignature;
}

pub(crate) struct InferenceSessionSet<K: Enum, R: InferenceRuntime>(
    EnumMap<K, Arc<std::sync::Mutex<R::Session>>>,
);

impl<K: Enum + Copy, R: InferenceRuntime> InferenceSessionSet<K, R> {
    pub(crate) fn new(
        model_bytes: &EnumMap<K, Vec<u8>>,
        mut options: impl FnMut(K) -> InferenceSessionOptions,
    ) -> anyhow::Result<Self> {
        let mut sessions = model_bytes
            .iter()
            .map(|(k, m)| {
                let sess = R::Session::new(|| model_file::decrypt(m), options(k))?;
                Ok((k.into_usize(), std::sync::Mutex::new(sess).into()))
            })
            .collect::<anyhow::Result<HashMap<_, _>>>()?;

        Ok(Self(EnumMap::<K, _>::from_fn(|k| {
            sessions.remove(&k.into_usize()).expect("should exist")
        })))
    }
}

impl<K: Enum, R: InferenceRuntime> InferenceSessionSet<K, R> {
    pub(crate) fn get<I>(&self) -> InferenceSessionCell<R, I>
    where
        I: InferenceInput,
        I::Signature: InferenceSignature<Kind = K>,
    {
        InferenceSessionCell {
            inner: self.0[<I::Signature as InferenceSignature>::KIND].clone(),
            marker: PhantomData,
        }
    }
}

pub(crate) struct InferenceSessionCell<R: InferenceRuntime, I> {
    inner: Arc<std::sync::Mutex<R::Session>>,
    marker: PhantomData<fn(I)>,
}

impl<
        R: SupportsInferenceInputTensors<I>
            + SupportsInferenceOutput<<I::Signature as InferenceSignature>::Output>,
        I: InferenceInput,
    > InferenceSessionCell<R, I>
{
    pub(crate) fn run(
        self,
        input: I,
    ) -> crate::Result<<I::Signature as InferenceSignature>::Output> {
        let mut inner = self.inner.lock().unwrap();
        let mut ctx = R::RunContext::from(&mut inner);
        R::input(&mut ctx, input);
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
