pub(crate) mod domain;
mod model_file;
pub(crate) mod runtimes;
pub(crate) mod status;

use std::{borrow::Cow, fmt::Debug};

use derive_new::new;
use enum_map::{Enum, EnumMap};
use ndarray::{Array, ArrayD, Dimension, ShapeError};
use thiserror::Error;

use crate::SupportedDevices;

pub(crate) trait InferenceRuntime: 'static {
    type Session: Sized + Send + 'static;
    type RunContext<'a>: From<&'a mut Self::Session>;

    fn supported_devices() -> crate::Result<SupportedDevices>;

    #[allow(clippy::type_complexity)]
    fn new_session(
        model: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        options: InferenceSessionOptions,
    ) -> anyhow::Result<(
        Self::Session,
        Vec<ParamInfo<InputScalarKind>>,
        Vec<ParamInfo<OutputScalarKind>>,
    )>;

    fn push_input(
        input: Array<impl InputScalar, impl Dimension + 'static>,
        ctx: &mut Self::RunContext<'_>,
    );

    fn run(ctx: Self::RunContext<'_>) -> anyhow::Result<Vec<OutputTensor>>;
}

/// ある`VoiceModel`が提供する推論操作の集合を示す。
pub(crate) trait InferenceDomain {
    type Operation: InferenceOperation;
}

/// `InferenceDomain`の推論操作を表す列挙型。
///
/// それぞれのバリアントには、対応する`InferenceSignature`が存在するべきである。
///
/// `::macros::InferenceOperation`により導出される。
pub(crate) trait InferenceOperation: Copy + Enum {
    /// `{InferenceInputSignature,InferenceOutputSignature}::PARAM_INFOS`を集めたもの。
    ///
    /// マクロ（voicevox_core_macros）で実装される前提。
    #[allow(clippy::type_complexity)]
    const PARAM_INFOS: EnumMap<
        Self,
        (
            &'static [ParamInfo<InputScalarKind>],
            &'static [ParamInfo<OutputScalarKind>],
        ),
    >;
}

/// `InferenceDomain`の推論操作を表す列挙型。
///
/// `::macros::InferenceOperation`により、具体型ごと生成される。
pub(crate) trait InferenceSignature: Sized + Send + 'static {
    type Domain: InferenceDomain;
    type Input: InferenceInputSignature<Signature = Self>;
    type Output: InferenceOutputSignature;
    const OPERATION: <Self::Domain as crate::infer::InferenceDomain>::Operation;
}

/// 推論操作の入力シグネチャ。
///
/// `::macros::InferenceInputSignature`により導出される。
pub(crate) trait InferenceInputSignature: Send + 'static {
    type Signature: InferenceSignature<Input = Self>;
    const PARAM_INFOS: &'static [ParamInfo<InputScalarKind>];
    fn make_run_context<R: InferenceRuntime>(self, sess: &mut R::Session) -> R::RunContext<'_>;
}

pub(crate) trait InputScalar: sealed::InputScalar + Debug + 'static {
    const KIND: InputScalarKind;
}

impl InputScalar for i64 {
    const KIND: InputScalarKind = InputScalarKind::Int64;
}

impl InputScalar for f32 {
    const KIND: InputScalarKind = InputScalarKind::Float32;
}

#[derive(Clone, Copy, PartialEq, derive_more::Display)]
pub(crate) enum InputScalarKind {
    #[display(fmt = "int64_t")]
    Int64,

    #[display(fmt = "float")]
    Float32,
}

/// 推論操作の出力シグネチャ。
///
/// `::macros::InferenceOutputSignature`により、`TryFrom<OutputTensor>`も含めて導出される。
pub(crate) trait InferenceOutputSignature:
    TryFrom<Vec<OutputTensor>, Error = anyhow::Error> + Send
{
    const PARAM_INFOS: &'static [ParamInfo<OutputScalarKind>];
}

pub(crate) trait OutputScalar: Sized {
    const KIND: OutputScalarKind;
    fn extract(tensor: OutputTensor) -> std::result::Result<ArrayD<Self>, ExtractError>;
}

impl OutputScalar for f32 {
    const KIND: OutputScalarKind = OutputScalarKind::Float32;

    fn extract(tensor: OutputTensor) -> std::result::Result<ArrayD<Self>, ExtractError> {
        match tensor {
            OutputTensor::Float32(tensor) => Ok(tensor),
        }
    }
}

#[derive(Clone, Copy, PartialEq, derive_more::Display)]
pub(crate) enum OutputScalarKind {
    #[display(fmt = "float")]
    Float32,
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

pub(crate) struct ParamInfo<D> {
    name: Cow<'static, str>,
    dt: D,
    ndim: Option<usize>,
}

impl<D: PartialEq> ParamInfo<D> {
    fn accepts(&self, other: &Self) -> bool {
        self.name == other.name
            && self.dt == other.dt
            && (self.ndim.is_none() || self.ndim == other.ndim)
    }
}

pub(crate) trait ArrayExt {
    type Scalar;
    type Dimension: Dimension;
}

impl<A, D: Dimension> ArrayExt for Array<A, D> {
    type Scalar = A;
    type Dimension = D;
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

// FIXME: `onnxruntime::TypeToTensorElementDataType`に依存する代わりに、`InputScalar`から`runtimes`
// まではvisitor patternでつなぐ
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
