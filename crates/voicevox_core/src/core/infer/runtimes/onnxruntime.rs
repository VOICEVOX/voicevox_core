// TODO: `VoiceModelFile`のように、次のような設計にする。
//
// ```
// pub(crate) mod blocking {
//     pub struct Onnxruntime(Inner<SingleTasked>);
//     // …
// }
// pub(crate) mod nonblocking {
//     pub struct Onnxruntime(Inner<BlockingThreadPool>);
//     // …
// }
// ```

use std::{
    cmp,
    ffi::CStr,
    fmt::{Debug, Display},
    sync::{Arc, LazyLock},
    vec,
};

use anyhow::{anyhow, bail, ensure, Context as _};
use duplicate::duplicate_item;
use ndarray::{Array, Dimension};
use ort::{
    environment::Environment,
    execution_providers::{
        cuda::CuDNNConvAlgorithmSearch, CPUExecutionProvider, CUDAExecutionProvider,
        DirectMLExecutionProvider, ExecutionProvider as _,
    },
    session::{builder::GraphOptimizationLevel, RunOptions},
    tensor::{PrimitiveTensorElementType, TensorElementType},
    value::ValueType,
};
use tracing::warn;

use crate::error::ErrorRepr;

use super::super::{
    super::{
        devices::{DeviceSpec, GpuSpec, SupportedDevices},
        voice_model::ModelBytes,
    },
    InferenceRuntime, InferenceSessionOptions, InputScalarKind, OutputScalarKind, OutputTensor,
    ParamInfo, PushInputTensor,
};

static SINGLETON: once_cell::sync::OnceCell<Inner> = once_cell::sync::OnceCell::new();

#[derive(Debug)]
struct Inner {
    _env: &'static Environment, // TODO: `ort`をv2.0.0-rc.11にしたら外に追い出す

    #[cfg(feature = "load-onnxruntime")]
    _lib: libloading::Library,
}

impl Inner {
    fn get() -> Option<&'static Self> {
        SINGLETON.get()
    }

    #[cfg(feature = "load-onnxruntime")]
    fn get_or_try_init(filename: &std::ffi::OsStr) -> crate::Result<&'static Self> {
        use anyhow::Context as _;
        use libloading::Library;

        SINGLETON
            .get_or_try_init(|| {
                // SAFETY: Users' responsibility!
                let lib = unsafe { Library::new(filename)? };
                let ort_get_api_base = unsafe {
                    lib.get::<unsafe extern "C" fn() -> *const ort::sys::OrtApiBase>(
                        b"OrtGetApiBase",
                    )
                }
                .with_context(|| "`OrtGetApiBase` not found")?;

                // SAFETY: `OrtGetApiBase` should require no preconditions,
                // and should return a valid `OrtApiBase`.
                let api_base = unsafe { ({ ort_get_api_base })() };
                assert!(!api_base.is_null() && api_base.is_aligned());
                let api_base = unsafe { &*api_base };

                let _env = setup(
                    api_base,
                    #[cfg(windows)]
                    TargetLibOnnxruntime { dll: &lib },
                    #[cfg(not(windows))]
                    TargetLibOnnxruntime { filename },
                )?;

                Ok(Self { _env, _lib: lib })
            })
            .map_err(|source| {
                ErrorRepr::InitInferenceRuntime {
                    runtime_display_name: "ONNX Runtime",
                    source,
                }
                .into()
            })
    }

    #[cfg(not(feature = "load-onnxruntime"))]
    fn get_or_try_init() -> crate::Result<&'static Self> {
        use std::marker::PhantomData;

        SINGLETON
            .get_or_try_init(|| {
                // SAFETY: `OrtGetApiBase` should require no preconditions,
                // and should return a valid `OrtApiBase`.
                let api_base = unsafe { ort::sys::OrtGetApiBase() };
                assert!(!api_base.is_null() && api_base.is_aligned());
                let api_base = unsafe { &*api_base };

                let _env = setup(
                    api_base,
                    TargetLibOnnxruntime {
                        _marker: PhantomData,
                    },
                )?;
                Ok(Self { _env })
            })
            .map_err(|source| {
                ErrorRepr::InitInferenceRuntime {
                    runtime_display_name: "ONNX Runtime",
                    source,
                }
                .into()
            })
    }
}

fn setup(
    api_base: &ort::sys::OrtApiBase,
    lib: TargetLibOnnxruntime<'_>,
) -> anyhow::Result<&'static Environment> {
    const EXPECTED_MAJOR_VERSION: u64 = 1;

    const _: () = assert!(ort::sys::ORT_API_VERSION == 17);

    // SAFETY: `GetVersionString` should require no preconditions,
    // and should return a valid string.
    let version_string = unsafe { ((api_base).GetVersionString)() };
    let version_string = unsafe { CStr::from_ptr(version_string) };

    let (major_version, minor_version) = version_string
        .to_str()
        .ok()
        .and_then(|version_string| {
            let mut version_string = version_string.split('.');
            let major_version = version_string.next()?.parse::<u64>().ok()?;
            let minor_version = version_string
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            Some((major_version, minor_version))
        })
        .with_context(|| "could not parse the version string")?;

    ensure!(
        major_version == EXPECTED_MAJOR_VERSION,
        "invalid major version",
    );

    match minor_version.cmp(&u64::from(ort::sys::ORT_API_VERSION)) {
        cmp::Ordering::Less => bail!(
            "{message_for_version}。\
             ONNX Runtimeはバージョン1.{ORT_API_VERSION}でなくてはなりません",
            message_for_version = lib.message_for_version(version_string.to_string_lossy()),
            ORT_API_VERSION = ort::sys::ORT_API_VERSION,
        ),
        // TODO: 問題ないとわかっている既知のものであれば警告無しで許容しつつ、未来のものは拒否する。
        cmp::Ordering::Greater => warn!(
            "{message_for_version}。\
             対応しているONNX Runtimeのバージョンは1.{ORT_API_VERSION}なので、\
             互換性の問題があるかもしれません",
            message_for_version = lib.message_for_version(version_string.to_string_lossy()),
            ORT_API_VERSION = ort::sys::ORT_API_VERSION,
        ),
        cmp::Ordering::Equal => {}
    };

    // SAFETY: `GetApi` should require no preconditions, and should return a valid `OrtApi`.
    let api = unsafe { (api_base.GetApi)(ort::sys::ORT_API_VERSION) };
    assert!(!api.is_null() && api.is_aligned());
    let api = unsafe { api.read() };

    let inserted = ort::set_api(api);
    if !inserted {
        warn!("`ort::set_api` already executed");
    }

    let configured = ort::init()
        .with_name(env!("CARGO_PKG_NAME"))
        .commit()
        .expect("could not create a ORT environment"); // v2.0.0-rc.10だと、コケた時点でUBであるため
    if !configured {
        warn!("`ort::environment::EnvironmentBuilder` was already configured");
    }

    Ok(ort::environment::get_environment().expect("should have been already set"))
}

#[derive(Clone, Copy)]
struct TargetLibOnnxruntime<'a> {
    #[cfg(all(feature = "load-onnxruntime", windows))]
    dll: &'a libloading::Library,

    #[cfg(all(feature = "load-onnxruntime", not(windows)))]
    filename: &'a std::ffi::OsStr,

    #[cfg(not(feature = "load-onnxruntime"))]
    _marker: std::marker::PhantomData<&'a ()>,
}

impl TargetLibOnnxruntime<'_> {
    #[cfg(all(feature = "load-onnxruntime", windows))]
    fn message_for_version(self, version_string: impl Display) -> String {
        let Self { dll } = self;
        format!("`{dll:?}`はバージョン{version_string}のONNX Runtimeです")
    }

    #[cfg(all(feature = "load-onnxruntime", not(windows)))]
    fn message_for_version(self, version_string: impl Display) -> String {
        let Self { filename } = self;
        let filename = filename.to_string_lossy();
        format!("`{filename}`で指定されたONNX Runtimeはバージョン{version_string}です")
    }

    #[cfg(not(feature = "load-onnxruntime"))]
    fn message_for_version(self, version_string: impl Display) -> String {
        let Self { _marker } = self;
        format!("リンクされたONNX Runtimeはバージョン{version_string}です")
    }
}

impl InferenceRuntime for self::blocking::Onnxruntime {
    type Session = async_lock::Mutex<ort::session::Session>; // WASMでは`ort`を利用しないので、ここはasync-lockを用いてよいはず
    type RunContext = OnnxruntimeRunContext;

    const DISPLAY_NAME: &'static str = if cfg!(feature = "load-onnxruntime") {
        "現在ロードされているONNX Runtime"
    } else if cfg!(feature = "link-onnxruntime") {
        "現在リンクされているONNX Runtime"
    } else {
        panic!("either `load-onnxruntime` or `link-onnxruntime` must be enabled");
    };

    fn supported_devices(&self) -> crate::Result<SupportedDevices> {
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

    fn test_gpu(&self, gpu: GpuSpec) -> anyhow::Result<()> {
        let sess_builder = &mut ort::session::builder::SessionBuilder::new()?;
        match gpu {
            GpuSpec::Cuda => CUDAExecutionProvider::default()
                .with_conv_algorithm_search(CuDNNConvAlgorithmSearch::Default)
                .register(sess_builder),
            GpuSpec::Dml => DirectMLExecutionProvider::default().register(sess_builder),
        }
        .map_err(Into::into)
    }

    fn new_session(
        &self,
        model: &ModelBytes,
        options: InferenceSessionOptions,
    ) -> anyhow::Result<(
        Self::Session,
        Vec<ParamInfo<InputScalarKind>>,
        Vec<ParamInfo<OutputScalarKind>>,
    )> {
        static IS_VOICEVOX_ONNXRUNTIME: LazyLock<bool> =
            LazyLock::new(|| ort::info().starts_with("VOICEVOX ORT Build Info: "));

        let mut builder = ort::session::Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level1)?
            .with_intra_threads(options.cpu_num_threads.into())?;

        match options.device {
            DeviceSpec::Cpu => {}
            DeviceSpec::Gpu(GpuSpec::Cuda) => {
                CUDAExecutionProvider::default()
                    .with_conv_algorithm_search(CuDNNConvAlgorithmSearch::Default)
                    .register(&mut builder)?;
            }
            DeviceSpec::Gpu(GpuSpec::Dml) => {
                builder = builder
                    .with_parallel_execution(false)?
                    .with_memory_pattern(false)?;
                DirectMLExecutionProvider::default().register(&mut builder)?;
            }
        };

        let sess = match model {
            ModelBytes::Onnx(onnx) => builder.commit_from_memory(onnx),
            ModelBytes::VvBin(bin) => {
                ensure!(
                    *IS_VOICEVOX_ONNXRUNTIME,
                    "This ONNX Runtime does not support \"vv-bin\" format \
                     (note: load/link `voicevox_onnxruntime` instead of ` onnxruntime`)",
                );
                builder
                    .with_config_entry("session.use_vv_bin", "1")?
                    .commit_from_memory(bin)
            }
        }?;

        let input_param_infos = sess
            .inputs
            .iter()
            .map(|info| {
                let ValueType::Tensor { ty, .. } = info.input_type else {
                    bail!(
                        "unexpected input value type for `{}`. currently `ONNX_TYPE_TENSOR` and \
                         `ONNX_TYPE_SPARSETENSOR` is supported",
                        info.name,
                    );
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
                    TensorElementType::Bfloat16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_BFLOAT16"),
                    TensorElementType::Float16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_FLOAT16"),
                    TensorElementType::Float64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_DOUBLE"),
                    TensorElementType::Uint32 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT32"),
                    TensorElementType::Uint64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT64"),
                    TensorElementType::Bool => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_BOOL"),
                    TensorElementType::Complex64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_COMPLEX64"),
                    TensorElementType::Complex128 => {
                        Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_COMPLEX128")
                    }
                    TensorElementType::Float8E4M3FN => {
                        Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_FLOAT8E4M3FN")
                    }
                    TensorElementType::Float8E4M3FNUZ => {
                        Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_FLOAT8E4M3FNUZ")
                    }
                    TensorElementType::Float8E5M2 => {
                        Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_FLOAT8E5M2")
                    }
                    TensorElementType::Float8E5M2FNUZ => {
                        Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_FLOAT8E5M2FNUZ")
                    }
                    TensorElementType::Undefined => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UNDEFINED"),
                }
                .map_err(|actual| {
                    anyhow!("unsupported input datatype `{actual}` for `{}`", info.name)
                })?;

                Ok(ParamInfo {
                    name: info.name.clone().into(),
                    dt,
                    ndim: info.input_type.tensor_shape().map(|s| s.len()),
                })
            })
            .collect::<anyhow::Result<_>>()?;

        let output_param_infos = sess
            .outputs
            .iter()
            .map(|info| {
                let ValueType::Tensor { ty, .. } = info.output_type else {
                    bail!(
                        "unexpected output value type for `{}`. currently `ONNX_TYPE_TENSOR` and \
                         `ONNX_TYPE_SPARSETENSOR` is supported",
                        info.name,
                    );
                };

                let dt = match ty {
                    TensorElementType::Float32 => Ok(OutputScalarKind::Float32),
                    TensorElementType::Uint8 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT8"),
                    TensorElementType::Int8 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT8"),
                    TensorElementType::Uint16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT16"),
                    TensorElementType::Int16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT16"),
                    TensorElementType::Int32 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT32"),
                    TensorElementType::Int64 => Ok(OutputScalarKind::Int64),
                    TensorElementType::String => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_STRING"),
                    TensorElementType::Bfloat16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_BFLOAT16"),
                    TensorElementType::Float16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_FLOAT16"),
                    TensorElementType::Float64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_DOUBLE"),
                    TensorElementType::Uint32 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT32"),
                    TensorElementType::Uint64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT64"),
                    TensorElementType::Bool => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_BOOL"),
                    TensorElementType::Complex64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_COMPLEX64"),
                    TensorElementType::Complex128 => {
                        Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_COMPLEX128")
                    }
                    TensorElementType::Float8E4M3FN => {
                        Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_FLOAT8E4M3FN")
                    }
                    TensorElementType::Float8E4M3FNUZ => {
                        Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_FLOAT8E4M3FNUZ")
                    }
                    TensorElementType::Float8E5M2 => {
                        Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_FLOAT8E5M2")
                    }
                    TensorElementType::Float8E5M2FNUZ => {
                        Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_FLOAT8E5M2FNUZ")
                    }
                    TensorElementType::Undefined => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UNDEFINED"),
                }
                .map_err(|actual| {
                    anyhow!("unsupported output datatype `{actual}` for `{}`", info.name)
                })?;

                Ok(ParamInfo {
                    name: info.name.clone().into(),
                    dt,
                    ndim: info.output_type.tensor_shape().map(|s| s.len()),
                })
            })
            .collect::<anyhow::Result<_>>()?;

        Ok((sess.into(), input_param_infos, output_param_infos))
    }

    fn run_blocking(
        OnnxruntimeRunContext { sess, inputs }: Self::RunContext,
    ) -> anyhow::Result<Vec<OutputTensor>> {
        extract_outputs(&sess.lock_blocking().run(inputs)?)
    }

    async fn run_async(
        OnnxruntimeRunContext { sess, inputs }: Self::RunContext,
        cancellable: bool,
    ) -> anyhow::Result<Vec<OutputTensor>> {
        if cancellable {
            extract_outputs(
                &sess
                    .lock()
                    .await
                    .run_async(inputs, &RunOptions::new()?)?
                    .await?,
            )
        } else {
            ::blocking::unblock(move || extract_outputs(&sess.lock_blocking().run(inputs)?)).await
        }
    }
}

pub(crate) struct OnnxruntimeRunContext {
    sess: Arc<async_lock::Mutex<ort::session::Session>>,
    inputs: Vec<(&'static str, ort::session::SessionInputValue<'static>)>,
}

impl OnnxruntimeRunContext {
    fn push_input(
        &mut self,
        name: &'static str,
        input: Array<
            impl PrimitiveTensorElementType + Debug + Clone + 'static,
            impl Dimension + 'static,
        >,
    ) -> anyhow::Result<()> {
        let input = ort::value::Value::from_array(input)?.into();
        self.inputs.push((name, input));
        Ok(())
    }
}

impl From<Arc<async_lock::Mutex<ort::session::Session>>> for OnnxruntimeRunContext {
    fn from(sess: Arc<async_lock::Mutex<ort::session::Session>>) -> Self {
        Self {
            sess,
            inputs: vec![],
        }
    }
}

impl PushInputTensor for OnnxruntimeRunContext {
    #[duplicate_item(
        method           T;
        [ push_int64 ]   [ i64 ];
        [ push_float32 ] [ f32 ];
    )]
    fn method(
        &mut self,
        name: &'static str,
        tensor: Array<T, impl Dimension + 'static>,
    ) -> anyhow::Result<()> {
        self.push_input(name, tensor)
    }
}

// FIXME: use ouroboros to reduce copies
fn extract_outputs(
    outputs: &ort::session::SessionOutputs<'_>,
) -> anyhow::Result<Vec<OutputTensor>> {
    (0..outputs.len())
        .map(|i| {
            let output = &outputs[i];

            let ValueType::Tensor { ty, .. } = output.dtype() else {
                bail!(
                    "unexpected output. currently `ONNX_TYPE_TENSOR` and `ONNX_TYPE_SPARSETENSOR`
                     is supported",
                );
            };

            match ty {
                TensorElementType::Int64 => {
                    let output = output.try_extract_array::<i64>()?;
                    Ok(OutputTensor::Int64(output.into_owned()))
                }
                TensorElementType::Float32 => {
                    let output = output.try_extract_array::<f32>()?;
                    Ok(OutputTensor::Float32(output.into_owned()))
                }
                _ => bail!("unexpected output tensor element data type"),
            }
        })
        .collect()
}

pub(crate) mod blocking {
    use ref_cast::{ref_cast_custom, RefCastCustom};

    use crate::SupportedDevices;

    use super::{super::super::InferenceRuntime, Inner};

    /// ONNX Runtime。
    ///
    /// シングルトンであり、インスタンスは高々一つ。インスタンスは[非同期版の`Onnxruntime`]と共有される。
    ///
    /// [非同期版の`Onnxruntime`]: crate::nonblocking::Onnxruntime
    #[cfg_attr(feature = "load-onnxruntime", doc = "```")]
    #[cfg_attr(not(feature = "load-onnxruntime"), doc = "```compile_fail")]
    /// # fn main() -> anyhow::Result<()> {
    /// # voicevox_core::blocking::Onnxruntime::load_once()
    /// #     .filename(test_util::ONNXRUNTIME_DYLIB_PATH)
    /// #     .perform()?;
    /// #
    /// use std::ptr;
    ///
    /// let ort1 = voicevox_core::blocking::Onnxruntime::load_once().perform()?;
    /// let ort2 = voicevox_core::nonblocking::Onnxruntime::get().expect("`ort1`と同一のはず");
    /// assert!(ptr::addr_eq(ort1, ort2));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [voicevox-ort]: https://github.com/VOICEVOX/ort
    #[cfg_attr(doc, doc(alias = "VoicevoxOnnxruntime"))]
    #[derive(Debug, RefCastCustom)]
    #[repr(transparent)]
    pub struct Onnxruntime(Inner);

    impl Onnxruntime {
        /// ONNX Runtimeのライブラリ名。
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        pub const LIB_NAME: &'static str = "voicevox_onnxruntime";

        /// 推奨されるONNX Runtimeのバージョン。
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        pub const LIB_VERSION: &'static str = include_str!("../../../../onnxruntime-version.txt");

        /// [`LIB_NAME`]と[`LIB_VERSION`]からなる動的ライブラリのファイル名。
        ///
        /// WindowsとAndroidでは[`LIB_UNVERSIONED_FILENAME`]と同じ。
        ///
        /// [`LIB_NAME`]: Self::LIB_NAME
        /// [`LIB_VERSION`]: Self::LIB_VERSION
        /// [`LIB_UNVERSIONED_FILENAME`]: Self::LIB_UNVERSIONED_FILENAME
        #[cfg_attr(doc, doc(alias = "voicevox_get_onnxruntime_lib_versioned_filename"))]
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        pub const LIB_VERSIONED_FILENAME: &'static str = if cfg!(target_os = "linux") {
            const_format::concatcp!(
                "lib",
                Onnxruntime::LIB_NAME,
                ".so.",
                Onnxruntime::LIB_VERSION,
            )
        } else if cfg!(any(target_os = "macos", target_os = "ios")) {
            const_format::concatcp!(
                "lib",
                Onnxruntime::LIB_NAME,
                ".",
                Onnxruntime::LIB_VERSION,
                ".dylib",
            )
        } else {
            Self::LIB_UNVERSIONED_FILENAME
        };

        /// [`LIB_NAME`]からなる動的ライブラリのファイル名。
        ///
        /// [`LIB_NAME`]: Self::LIB_NAME
        #[cfg_attr(doc, doc(alias = "voicevox_get_onnxruntime_lib_unversioned_filename"))]
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        pub const LIB_UNVERSIONED_FILENAME: &'static str = const_format::concatcp!(
            std::env::consts::DLL_PREFIX,
            Onnxruntime::LIB_NAME,
            std::env::consts::DLL_SUFFIX,
        );

        #[ref_cast_custom]
        const fn new(inner: &Inner) -> &Self;

        /// インスタンスが既に作られているならそれを得る。
        ///
        /// 作られていなければ`None`を返す。
        #[cfg_attr(doc, doc(alias = "voicevox_onnxruntime_get"))]
        pub fn get() -> Option<&'static Self> {
            Inner::get().map(Self::new)
        }

        /// ONNX Runtimeをロードして初期化する。
        ///
        /// 一度成功したら、以後は引数を無視して同じ参照を返す。
        #[cfg_attr(doc, doc(alias = "voicevox_onnxruntime_load_once"))]
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        pub fn load_once() -> LoadOnce {
            LoadOnce::default()
        }

        /// ONNX Runtimeを初期化する。
        ///
        /// 一度成功したら以後は同じ参照を返す。
        #[cfg_attr(doc, doc(alias = "voicevox_onnxruntime_init_once"))]
        #[cfg(feature = "link-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "link-onnxruntime")))]
        pub fn init_once() -> crate::Result<&'static Self> {
            Inner::get_or_try_init().map(Onnxruntime::new)
        }

        #[cfg(test)]
        pub(crate) fn from_test_util_data() -> anyhow::Result<&'static Self> {
            #[cfg(feature = "load-onnxruntime")]
            {
                Self::load_once()
                    .filename(test_util::ONNXRUNTIME_DYLIB_PATH)
                    .perform()
                    .map_err(Into::into)
            }

            #[cfg(feature = "link-onnxruntime")]
            {
                Self::init_once().map_err(Into::into)
            }
        }

        /// ONNX Runtimeとして利用可能なデバイスの情報を取得する。
        #[cfg_attr(doc, doc(alias = "voicevox_onnxruntime_create_supported_devices_json"))]
        pub fn supported_devices(&self) -> crate::Result<SupportedDevices> {
            <Self as InferenceRuntime>::supported_devices(self)
        }
    }

    /// [`Onnxruntime::load_once`]のビルダー。
    #[cfg(feature = "load-onnxruntime")]
    #[must_use = "this is a builder. it does nothing until `perform`ed"]
    #[derive(Debug)]
    pub struct LoadOnce {
        filename: std::ffi::OsString,
    }

    #[cfg(feature = "load-onnxruntime")]
    impl Default for LoadOnce {
        fn default() -> Self {
            let filename = Onnxruntime::LIB_VERSIONED_FILENAME.into();
            Self { filename }
        }
    }

    #[cfg(feature = "load-onnxruntime")]
    impl LoadOnce {
        /// ONNX Runtimeのファイル名（モジュール名）もしくはファイルパスを指定する。
        ///
        /// `dlopen`/[`LoadLibraryExW`]の引数に使われる。デフォルトは[`Onnxruntime::LIB_VERSIONED_FILENAME`]。
        ///
        /// [`LoadLibraryExW`]:
        /// https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw
        pub fn filename(mut self, filename: impl Into<std::ffi::OsString>) -> Self {
            self.filename = filename.into();
            self
        }

        /// 実行する。
        pub fn perform(self) -> crate::Result<&'static Onnxruntime> {
            Inner::get_or_try_init(&self.filename).map(Onnxruntime::new)
        }
    }
}

pub(crate) mod nonblocking {
    use ref_cast::{ref_cast_custom, RefCastCustom};

    use crate::SupportedDevices;

    /// ONNX Runtime。
    ///
    /// シングルトンであり、インスタンスは高々一つ。インスタンスは[ブロッキング版の`Onnxruntime`]と共有される。
    ///
    /// [ブロッキング版の`Onnxruntime`]: crate::blocking::Onnxruntime
    #[cfg_attr(feature = "load-onnxruntime", doc = "```")]
    #[cfg_attr(not(feature = "load-onnxruntime"), doc = "```compile_fail")]
    /// # #[pollster::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # voicevox_core::blocking::Onnxruntime::load_once()
    /// #     .filename(test_util::ONNXRUNTIME_DYLIB_PATH)
    /// #     .perform()?;
    /// #
    /// use std::ptr;
    ///
    /// let ort1 = voicevox_core::nonblocking::Onnxruntime::load_once()
    ///     .perform()
    ///     .await?;
    /// let ort2 = voicevox_core::blocking::Onnxruntime::get().expect("`ort1`と同一のはず");
    /// assert!(ptr::addr_eq(ort1, ort2));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance
    ///
    /// [blocking]クレートにより動いている。詳しくは[`nonblocking`モジュールのドキュメント]を参照。
    ///
    /// [voicevox-ort]: https://github.com/VOICEVOX/ort
    /// [blocking]: https://docs.rs/crate/blocking
    /// [`nonblocking`モジュールのドキュメント]: crate::nonblocking
    #[derive(Debug, RefCastCustom)]
    #[repr(transparent)]
    pub struct Onnxruntime(pub(crate) super::blocking::Onnxruntime);

    impl Onnxruntime {
        /// ONNX Runtimeのライブラリ名。
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        // ブロッキング版と等しいことはテストで担保
        pub const LIB_NAME: &'static str = "voicevox_onnxruntime";

        /// 推奨されるONNX Runtimeのバージョン。
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        // ブロッキング版と等しいことはテストで担保
        pub const LIB_VERSION: &'static str = include_str!("../../../../onnxruntime-version.txt");

        /// [`LIB_NAME`]と[`LIB_VERSION`]からなる動的ライブラリのファイル名。
        ///
        /// WindowsとAndroidでは[`LIB_UNVERSIONED_FILENAME`]と同じ。
        ///
        /// [`LIB_NAME`]: Self::LIB_NAME
        /// [`LIB_VERSION`]: Self::LIB_VERSION
        /// [`LIB_UNVERSIONED_FILENAME`]: Self::LIB_UNVERSIONED_FILENAME
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        pub const LIB_VERSIONED_FILENAME: &'static str =
            super::blocking::Onnxruntime::LIB_VERSIONED_FILENAME;

        /// [`LIB_NAME`]からなる動的ライブラリのファイル名。
        ///
        /// [`LIB_NAME`]: Self::LIB_NAME
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        pub const LIB_UNVERSIONED_FILENAME: &'static str =
            super::blocking::Onnxruntime::LIB_UNVERSIONED_FILENAME;

        #[ref_cast_custom]
        pub(crate) const fn from_blocking(blocking: &super::blocking::Onnxruntime) -> &Self;

        /// インスタンスが既に作られているならそれを得る。
        ///
        /// 作られていなければ`None`を返す。
        pub fn get() -> Option<&'static Self> {
            super::blocking::Onnxruntime::get().map(Self::from_blocking)
        }

        /// ONNX Runtimeをロードして初期化する。
        ///
        /// 一度成功したら、以後は引数を無視して同じ参照を返す。
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        pub fn load_once() -> LoadOnce {
            LoadOnce::default()
        }

        /// ONNX Runtimeを初期化する。
        ///
        /// 一度成功したら以後は同じ参照を返す。
        #[cfg(feature = "link-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "link-onnxruntime")))]
        pub async fn init_once() -> crate::Result<&'static Self> {
            let inner = crate::task::asyncify(super::blocking::Onnxruntime::init_once).await?;
            Ok(Self::from_blocking(inner))
        }

        #[cfg(test)]
        pub(crate) async fn from_test_util_data() -> anyhow::Result<&'static Self> {
            crate::task::asyncify(super::blocking::Onnxruntime::from_test_util_data)
                .await
                .map(Self::from_blocking)
        }

        /// ONNX Runtimeとして利用可能なデバイスの情報を取得する。
        pub fn supported_devices(&self) -> crate::Result<SupportedDevices> {
            self.0.supported_devices()
        }
    }

    /// [`Onnxruntime::load_once`]のビルダー。
    #[cfg(feature = "load-onnxruntime")]
    #[derive(Default, derive_more::Debug)]
    #[debug("{_0:?}")]
    #[must_use = "this is a builder. it does nothing until `perform`ed"]
    pub struct LoadOnce(super::blocking::LoadOnce);

    #[cfg(feature = "load-onnxruntime")]
    impl LoadOnce {
        /// ONNX Runtimeのファイル名（モジュール名）もしくはファイルパスを指定する。
        ///
        /// `dlopen`/[`LoadLibraryExW`]の引数に使われる。デフォルトは[`Onnxruntime::LIB_VERSIONED_FILENAME`]。
        ///
        /// [`LoadLibraryExW`]:
        /// https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw
        pub fn filename(self, filename: impl Into<std::ffi::OsString>) -> Self {
            Self(self.0.filename(filename))
        }

        /// 実行する。
        pub async fn perform(self) -> crate::Result<&'static Onnxruntime> {
            let inner = crate::task::asyncify(|| self.0.perform()).await?;
            Ok(Onnxruntime::from_blocking(inner))
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[cfg(feature = "load-onnxruntime")]
    #[test]
    fn assert_same_lib_names_and_versions() {
        use pretty_assertions::assert_eq;

        assert_eq!(
            super::blocking::Onnxruntime::LIB_NAME,
            super::nonblocking::Onnxruntime::LIB_NAME,
        );
        assert_eq!(
            super::blocking::Onnxruntime::LIB_VERSION,
            super::nonblocking::Onnxruntime::LIB_VERSION,
        );
    }

    #[rstest]
    fn supported_devices_works() {
        let result = super::blocking::Onnxruntime::from_test_util_data()
            .and_then(|o| o.supported_devices().map_err(Into::into));
        // 環境によって結果が変わるので、関数呼び出しが成功するかどうかの確認のみ行う
        assert!(result.is_ok(), "{result:?}");
    }
}
