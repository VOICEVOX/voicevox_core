use std::{fmt::Debug, vec};

use anyhow::{anyhow, bail, ensure};
use duplicate::duplicate_item;
use ndarray::{Array, Dimension};
use ort::{
    CPUExecutionProvider, CUDAExecutionProvider, DirectMLExecutionProvider, ExecutionProvider as _,
    GraphOptimizationLevel, IntoTensorElementType, TensorElementType, ValueType,
};

use crate::{devices::SupportedDevices, error::ErrorRepr};

use super::super::{
    DecryptModelError, InferenceRuntime, InferenceSessionOptions, InputScalarKind,
    OutputScalarKind, OutputTensor, ParamInfo, PushInputTensor,
};

// TODO: `trait AsyncRuntime`みたいなものを作って抽象化しながら同期版と非同期版に別個の役割を
// 持たせる
// （なぜそうしたいかの理由の一つとしては<https://github.com/VOICEVOX/voicevox_core/issues/687>）
impl InferenceRuntime for self::blocking::Onnxruntime {
    type Session = ort::Session;
    type RunContext<'a> = OnnxruntimeRunContext<'a>;

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

    fn new_session(
        &self,
        model: impl FnOnce() -> std::result::Result<Vec<u8>, DecryptModelError>,
        options: InferenceSessionOptions,
    ) -> anyhow::Result<(
        Self::Session,
        Vec<ParamInfo<InputScalarKind>>,
        Vec<ParamInfo<OutputScalarKind>>,
    )> {
        let mut builder = ort::Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level1)?
            .with_intra_threads(options.cpu_num_threads.into())?;

        if options.use_gpu && cfg!(feature = "directml") {
            builder = builder
                .with_parallel_execution(false)?
                .with_memory_pattern(false)?;
            DirectMLExecutionProvider::default().register(&builder)?;
        } else if options.use_gpu && cfg!(feature = "cuda") {
            CUDAExecutionProvider::default().register(&builder)?;
        }

        let model = model()?;
        let sess = builder.commit_from_memory(&{ model })?;

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
                }
                .map_err(|actual| {
                    anyhow!("unsupported input datatype `{actual}` for `{}`", info.name)
                })?;

                Ok(ParamInfo {
                    name: info.name.clone().into(),
                    dt,
                    ndim: info.input_type.tensor_dimensions().map(Vec::len),
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
                    TensorElementType::Int64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_INT64"),
                    TensorElementType::String => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_STRING"),
                    TensorElementType::Bfloat16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_BFLOAT16"),
                    TensorElementType::Float16 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_FLOAT16"),
                    TensorElementType::Float64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_DOUBLE"),
                    TensorElementType::Uint32 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT32"),
                    TensorElementType::Uint64 => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_UINT64"),
                    TensorElementType::Bool => Err("ONNX_TENSOR_ELEMENT_DATA_TYPE_BOOL"),
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

                let ValueType::Tensor { ty, .. } = output.dtype()? else {
                    bail!(
                        "unexpected output. currently `ONNX_TYPE_TENSOR` and \
                         `ONNX_TYPE_SPARSETENSOR` is supported",
                    );
                };

                match ty {
                    TensorElementType::Float32 => {
                        let output = output.try_extract_tensor::<f32>()?;
                        Ok(OutputTensor::Float32(output.into_owned()))
                    }
                    _ => bail!("unexpected output tensor element data type"),
                }
            })
            .collect()
    }
}

pub(crate) struct OnnxruntimeRunContext<'sess> {
    sess: &'sess ort::Session,
    inputs: Vec<ort::SessionInputValue<'static>>,
}

impl OnnxruntimeRunContext<'_> {
    fn push_input(
        &mut self,
        input: Array<
            impl IntoTensorElementType + Debug + Clone + 'static,
            impl Dimension + 'static,
        >,
    ) -> anyhow::Result<()> {
        let input = ort::Value::from_array(input)?.into();
        self.inputs.push(input);
        Ok(())
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
    fn method(&mut self, tensor: Array<T, impl Dimension + 'static>) -> anyhow::Result<()> {
        self.push_input(tensor)
    }
}

pub(crate) mod blocking {
    use ort::EnvHandle;
    use ref_cast::{ref_cast_custom, RefCastCustom};

    use crate::{error::ErrorRepr, SupportedDevices};

    use super::super::super::InferenceRuntime;

    /// ONNX Runtime。
    ///
    /// シングルトンであり、インスタンスは高々一つ。
    ///
    /// # Rust APIにおけるインスタンスの共有
    ///
    /// インスタンスは[voicevox-ort]側に作られる。Rustのクレートとしてこのライブラリを利用する場合、
    /// Tokio版APIやvoicevox-ortを利用する他クレートともインスタンスが共有される。
    ///
    /// ```
    /// # use voicevox_core as another_lib;
    /// #
    /// # fn main() -> anyhow::Result<()> {
    /// # if cfg!(windows) {
    /// #     // Windows\System32\onnxruntime.dllを回避
    /// #     voicevox_core::blocking::Onnxruntime::load_once()
    /// #         .filename(test_util::ONNXRUNTIME_DYLIB_PATH)
    /// #         .exec()?;
    /// # }
    /// let ort1 = voicevox_core::blocking::Onnxruntime::load_once().exec()?;
    /// let ort2 = another_lib::tokio::Onnxruntime::get().expect("`ort1`と同一のはず");
    /// assert_eq!(ptr_addr(ort1), ptr_addr(ort2));
    ///
    /// fn ptr_addr(obj: &impl Sized) -> usize {
    ///     obj as *const _ as _
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [voicevox-ort]: https://github.com/VOICEVOX/ort
    #[derive(Debug, RefCastCustom)]
    #[repr(transparent)]
    pub struct Onnxruntime {
        _inner: EnvHandle,
    }

    impl Onnxruntime {
        /// ONNX Runtimeのライブラリ名。
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        pub const LIB_NAME: &'static str = "onnxruntime";

        /// 推奨されるONNX Runtimeのバージョン。
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        pub const LIB_VERSION: &'static str = ort::downloaded_version!();

        /// [`LIB_NAME`]と[`LIB_VERSION`]からなる動的ライブラリのファイル名。
        ///
        /// WindowsとAndroidでは[`LIB_UNVERSIONED_FILENAME`]と同じ。
        ///
        /// [`LIB_NAME`]: Self::LIB_NAME
        /// [`LIB_VERSION`]: Self::LIB_VERSION
        /// [`LIB_UNVERSIONED_FILENAME`]: Self::LIB_UNVERSIONED_FILENAME
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
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        pub const LIB_UNVERSIONED_FILENAME: &'static str = const_format::concatcp!(
            std::env::consts::DLL_PREFIX,
            Onnxruntime::LIB_NAME,
            std::env::consts::DLL_SUFFIX,
        );

        #[ref_cast_custom]
        const fn new(inner: &EnvHandle) -> &Self;

        /// インスタンスが既に作られているならそれを得る。
        ///
        /// 作られていなければ`None`を返す。
        pub fn get() -> Option<&'static Self> {
            EnvHandle::get().map(Self::new)
        }

        fn once(
            init: impl FnOnce() -> anyhow::Result<&'static EnvHandle>,
        ) -> crate::Result<&'static Self> {
            let inner = init().map_err(|source| ErrorRepr::InitInferenceRuntime {
                runtime_display_name: "ONNX Runtime",
                source,
            })?;
            Ok(Self::new(inner))
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
        pub fn init_once() -> crate::Result<&'static Self> {
            Self::once(|| ort::try_init(None))
        }

        #[cfg(test)]
        pub(crate) fn from_test_util_data() -> anyhow::Result<&'static Self> {
            #[cfg(feature = "load-onnxruntime")]
            {
                Self::load_once()
                    .filename(test_util::ONNXRUNTIME_DYLIB_PATH)
                    .exec()
                    .map_err(Into::into)
            }

            #[cfg(feature = "link-onnxruntime")]
            {
                Self::init_once().map_err(Into::into)
            }
        }

        /// このライブラリで利用可能なデバイスの情報を取得する。
        pub fn supported_devices(&self) -> crate::Result<SupportedDevices> {
            <Self as InferenceRuntime>::supported_devices(self)
        }
    }

    /// [`Onnxruntime::load_once`]のビルダー。
    #[cfg(feature = "load-onnxruntime")]
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
        /// `dlopen`/[`LoadLibraryExW`]の引数に使われる。デフォルト
        /// は[`Onnxruntime::LIB_VERSIONED_FILENAME`]。
        ///
        /// [`LoadLibraryExW`]:
        /// https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw
        pub fn filename(mut self, filename: impl Into<std::ffi::OsString>) -> Self {
            self.filename = filename.into();
            self
        }

        /// 実行する。
        pub fn exec(self) -> crate::Result<&'static Onnxruntime> {
            Onnxruntime::once(|| ort::try_init_from(&self.filename, None))
        }
    }
}

pub(crate) mod tokio {
    use ref_cast::{ref_cast_custom, RefCastCustom};

    use crate::SupportedDevices;

    /// ONNX Runtime。
    ///
    /// シングルトンであり、インスタンスは高々一つ。
    ///
    /// # Rust APIにおけるインスタンスの共有
    ///
    /// インスタンスは[voicevox-ort]側に作られる。Rustのクレートとしてこのライブラリを利用する場合、
    /// ブロッキング版APIやvoicevox-ortを利用する他クレートともインスタンスが共有される。
    ///
    /// ```
    /// # use voicevox_core as another_lib;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # if cfg!(windows) {
    /// #     // Windows\System32\onnxruntime.dllを回避
    /// #     voicevox_core::blocking::Onnxruntime::load_once()
    /// #         .filename(test_util::ONNXRUNTIME_DYLIB_PATH)
    /// #         .exec()?;
    /// # }
    /// let ort1 = voicevox_core::tokio::Onnxruntime::load_once().exec().await?;
    /// let ort2 = another_lib::blocking::Onnxruntime::get().expect("`ort1`と同一のはず");
    /// assert_eq!(ptr_addr(ort1), ptr_addr(ort2));
    ///
    /// fn ptr_addr(obj: &impl Sized) -> usize {
    ///     obj as *const _ as _
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [voicevox-ort]: https://github.com/VOICEVOX/ort
    #[derive(Debug, RefCastCustom)]
    #[repr(transparent)]
    pub struct Onnxruntime(pub(crate) super::blocking::Onnxruntime);

    impl Onnxruntime {
        /// ONNX Runtimeのライブラリ名。
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        // ブロッキング版と等しいことはテストで担保
        pub const LIB_NAME: &'static str = "onnxruntime";

        /// 推奨されるONNX Runtimeのバージョン。
        #[cfg(feature = "load-onnxruntime")]
        #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
        // ブロッキング版と等しいことはテストで担保
        pub const LIB_VERSION: &'static str = ort::downloaded_version!();

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

        /// このライブラリで利用可能なデバイスの情報を取得する。
        pub fn supported_devices(&self) -> crate::Result<SupportedDevices> {
            self.0.supported_devices()
        }
    }

    /// [`Onnxruntime::load_once`]のビルダー。
    #[cfg(feature = "load-onnxruntime")]
    #[derive(Default)]
    pub struct LoadOnce(super::blocking::LoadOnce);

    #[cfg(feature = "load-onnxruntime")]
    impl LoadOnce {
        /// ONNX Runtimeのファイル名（モジュール名）もしくはファイルパスを指定する。
        ///
        /// `dlopen`/[`LoadLibraryExW`]の引数に使われる。デフォルト
        /// は[`Onnxruntime::LIB_VERSIONED_FILENAME`]。
        ///
        /// [`LoadLibraryExW`]:
        /// https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw
        pub fn filename(self, filename: impl Into<std::ffi::OsString>) -> Self {
            Self(self.0.filename(filename))
        }

        /// 実行する。
        pub async fn exec(self) -> crate::Result<&'static Onnxruntime> {
            let inner = crate::task::asyncify(|| self.0.exec()).await?;
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
            super::tokio::Onnxruntime::LIB_NAME,
        );
        assert_eq!(
            super::blocking::Onnxruntime::LIB_VERSION,
            super::tokio::Onnxruntime::LIB_VERSION,
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
