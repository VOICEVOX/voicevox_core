pub(crate) mod domains;
pub(crate) mod runtimes;
pub(crate) mod session_set;

use std::{borrow::Cow, collections::BTreeSet, fmt::Debug, sync::Arc};

use derive_new::new;
use duplicate::duplicate_item;
use enum_map::{Enum, EnumMap};
use ndarray::{Array, ArrayD, Dimension, ShapeError};
use thiserror::Error;

use crate::{
    asyncs::{Async, BlockingThreadPool, SingleTasked},
    devices::{DeviceSpec, GpuSpec},
    voice_model::ModelBytes,
    StyleType, SupportedDevices,
};

pub(crate) trait AsyncExt: Async {
    async fn run_session<R: InferenceRuntime>(
        ctx: R::RunContext,
        async_cancellable: bool,
    ) -> anyhow::Result<Vec<OutputTensor>>;
}

impl AsyncExt for SingleTasked {
    async fn run_session<R: InferenceRuntime>(
        ctx: R::RunContext,
        _: bool,
    ) -> anyhow::Result<Vec<OutputTensor>> {
        R::run_blocking(ctx)
    }
}

impl AsyncExt for BlockingThreadPool {
    async fn run_session<R: InferenceRuntime>(
        ctx: R::RunContext,
        async_cancellable: bool,
    ) -> anyhow::Result<Vec<OutputTensor>> {
        R::run_async(ctx, async_cancellable).await
    }
}

pub(crate) trait InferenceRuntime: 'static {
    // TODO: "session"とは何なのかを定め、ドキュメントを書く。`InferenceSessionSet`も同様。
    type Session;

    // 本当は`From<&'_ Self::Session>`としたいが、 rust-lang/rust#100013 が立ち塞がる
    type RunContext: From<Arc<Self::Session>> + PushInputTensor;

    /// 名前。
    const DISPLAY_NAME: &'static str;

    /// このランタイムで利用可能なデバイスの情報を取得する。
    fn supported_devices(&self) -> crate::Result<SupportedDevices>;

    /// GPUが実際に利用できそうかどうか判定する。
    fn test_gpu(&self, gpu: GpuSpec) -> anyhow::Result<()>;

    #[expect(
        clippy::type_complexity,
        reason = "ここを呼び出すのは現状一箇所なので、可読性が著しく落ちてはいないことを考えると\
                  別にこのままでいいはず"
    )]
    fn new_session(
        &self,
        model: &ModelBytes,
        options: InferenceSessionOptions,
    ) -> anyhow::Result<(
        Self::Session,
        Vec<ParamInfo<InputScalarKind>>,
        Vec<ParamInfo<OutputScalarKind>>,
    )>;

    fn run_blocking(ctx: Self::RunContext) -> anyhow::Result<Vec<OutputTensor>>;

    async fn run_async(
        ctx: Self::RunContext,
        cancellable: bool,
    ) -> anyhow::Result<Vec<OutputTensor>>;
}

/// 共に扱われるべき推論操作の集合を示す。
pub(crate) trait InferenceDomain: Sized {
    type Operation: InferenceOperation;
    type Manifest;

    /// 対応する`StyleType`。
    ///
    /// 複数の`InferenceDomain`に対応する`StyleType`があってもよい。
    ///
    /// また、どの`InferenceDomain`にも属さない`StyleType`があってもよい。そのような`StyleType`は
    /// 音声モデルのロード時に単に拒否されるべきである。
    fn style_types() -> &'static BTreeSet<StyleType>;
}

/// `InferenceDomain`の推論操作を表す列挙型。
///
/// それぞれのバリアントには、対応する`InferenceSignature`が存在するべきである。
///
/// `::macros::InferenceOperation`により導出される。
pub(crate) trait InferenceOperation: Copy + Enum {
    /// `{InferenceInputSignature,InferenceOutputSignature}::PARAM_INFOS`を集めたもの。
    #[expect(
        clippy::type_complexity,
        reason = "ここを参照するのは現状一箇所なので、可読性が著しく落ちてはいないことを考えると\
                  別にこのままでいいはず"
    )]
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
pub(crate) trait InferenceSignature {
    type Domain: InferenceDomain;
    type Input: InferenceInputSignature<Signature = Self>;
    type Output: InferenceOutputSignature;
    const OPERATION: <Self::Domain as InferenceDomain>::Operation;
}

/// 推論操作の入力シグネチャ。
///
/// `::macros::InferenceInputSignature`により導出される。
pub(crate) trait InferenceInputSignature {
    type Signature: InferenceSignature<Input = Self>;
    const PARAM_INFOS: &'static [ParamInfo<InputScalarKind>];
    fn make_run_context<R: InferenceRuntime>(
        self,
        sess: Arc<R::Session>,
    ) -> anyhow::Result<R::RunContext>;
}

pub(crate) trait InputScalar: Sized {
    const KIND: InputScalarKind;

    // TODO: `Array`ではなく`ArrayView`を取ることができるかもしれない
    fn push_tensor_to_ctx(
        name: &'static str,
        tensor: Array<Self, impl Dimension + 'static>,
        visitor: &mut impl PushInputTensor,
    ) -> anyhow::Result<()>;
}

#[duplicate_item(
    T       KIND_VAL                     push;
    [ i64 ] [ InputScalarKind::Int64 ]   [ push_int64 ];
    [ f32 ] [ InputScalarKind::Float32 ] [ push_float32 ];
)]
impl InputScalar for T {
    const KIND: InputScalarKind = KIND_VAL;

    fn push_tensor_to_ctx(
        name: &'static str,
        tensor: Array<Self, impl Dimension + 'static>,
        ctx: &mut impl PushInputTensor,
    ) -> anyhow::Result<()> {
        ctx.push(name, tensor)
    }
}

#[derive(Clone, Copy, PartialEq, derive_more::Display)]
pub(crate) enum InputScalarKind {
    #[display("int64_t")]
    Int64,

    #[display("float")]
    Float32,
}

pub(crate) trait PushInputTensor {
    fn push_int64(
        &mut self,
        name: &'static str,
        tensor: Array<i64, impl Dimension + 'static>,
    ) -> anyhow::Result<()>;

    fn push_float32(
        &mut self,
        name: &'static str,
        tensor: Array<f32, impl Dimension + 'static>,
    ) -> anyhow::Result<()>;
}

/// 推論操作の出力シグネチャ。
///
/// `::macros::InferenceOutputSignature`により、`TryFrom<OutputTensor>`も含めて導出される。
pub(crate) trait InferenceOutputSignature:
    TryFrom<Vec<OutputTensor>, Error = anyhow::Error>
{
    const PARAM_INFOS: &'static [ParamInfo<OutputScalarKind>];
}

pub(crate) trait OutputScalar: Sized {
    const KIND: OutputScalarKind;
    fn extract(tensor: OutputTensor) -> std::result::Result<ArrayD<Self>, ExtractError>;
}

#[duplicate_item(
    T        Kind;
    [ i64 ] [ Int64 ];
    [ f32 ] [ Float32 ];
)]
impl OutputScalar for T {
    const KIND: OutputScalarKind = OutputScalarKind::Kind;

    fn extract(tensor: OutputTensor) -> std::result::Result<ArrayD<Self>, ExtractError> {
        match tensor {
            OutputTensor::Kind(tensor) => Ok(tensor),
            _ => Err(ExtractError::Datatype),
        }
    }
}

#[derive(Clone, Copy, PartialEq, derive_more::Display)]
pub(crate) enum OutputScalarKind {
    #[display("int64_t")]
    Int64,

    #[display("float")]
    Float32,
}

pub(crate) enum OutputTensor {
    Int64(ArrayD<i64>),
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

#[derive(new, Clone, Copy, PartialEq, Debug)]
pub(crate) struct InferenceSessionOptions {
    pub(crate) cpu_num_threads: u16,
    pub(crate) device: DeviceSpec,
}

// TODO: `ShapeError`を直接扱い、データ型違いはパニックにすべきでは？
#[derive(Error, Debug)]
pub(crate) enum ExtractError {
    #[error("wrong datatype")]
    Datatype,

    #[error(transparent)]
    Shape(#[from] ShapeError),
}
