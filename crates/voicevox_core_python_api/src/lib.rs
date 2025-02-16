#![expect(
    non_local_definitions,
    reason = "PyO3を≧0.21.0にすることで解決する予定"
)]

use std::{
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
};

mod convert;
use self::convert::from_utf8_path;
use easy_ext::ext;
use log::{debug, warn};
use macros::pyproject_project_version;
use pyo3::{
    create_exception,
    exceptions::{PyException, PyKeyError, PyValueError},
    pyfunction, pymodule,
    types::{PyBytes, PyList, PyModule},
    wrap_pyfunction, Py, PyAny, PyObject, PyResult, PyTypeInfo, Python,
};
use voicevox_core::__internal::interop::raii::MaybeClosed;

#[pymodule]
#[pyo3(name = "_rust")]
fn rust(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    module.add("__version__", pyproject_project_version!())?;
    module.add_wrapped(wrap_pyfunction!(_validate_user_dict_word))?;
    module.add_wrapped(wrap_pyfunction!(_to_zenkaku))?;
    module.add_wrapped(wrap_pyfunction!(wav_from_s16le))?;

    add_exceptions(module)?;

    let blocking_module = PyModule::new(py, "voicevox_core._rust.blocking")?;
    blocking_module.add_class::<self::blocking::Synthesizer>()?;
    blocking_module.add_class::<self::blocking::Onnxruntime>()?;
    blocking_module.add_class::<self::blocking::OpenJtalk>()?;
    blocking_module.add_class::<self::blocking::VoiceModelFile>()?;
    blocking_module.add_class::<self::blocking::UserDict>()?;
    blocking_module.add_class::<self::blocking::AudioFeature>()?;
    module.add_and_register_submodule(blocking_module)?;

    let asyncio_module = PyModule::new(py, "voicevox_core._rust.asyncio")?;
    asyncio_module.add_class::<self::asyncio::Synthesizer>()?;
    asyncio_module.add_class::<self::asyncio::Onnxruntime>()?;
    asyncio_module.add_class::<self::asyncio::OpenJtalk>()?;
    asyncio_module.add_class::<self::asyncio::VoiceModelFile>()?;
    asyncio_module.add_class::<self::asyncio::UserDict>()?;
    module.add_and_register_submodule(asyncio_module)
}

#[ext]
impl PyModule {
    // https://github.com/PyO3/pyo3/issues/1517#issuecomment-808664021
    fn add_and_register_submodule(&self, module: &PyModule) -> PyResult<()> {
        let sys = self.py().import("sys")?;
        sys.getattr("modules")?.set_item(module.name()?, module)?;
        self.add_submodule(module)
    }
}

macro_rules! exceptions {
    ($($name:ident: $base:ty;)*) => {
        $(
            create_exception!(voicevox_core, $name, $base);
        )*

        fn add_exceptions(module: &PyModule) -> PyResult<()> {
            $(
                module.add(stringify!($name), module.py().get_type::<$name>())?;
            )*
            Ok(())
        }
    };
}

exceptions! {
    NotLoadedOpenjtalkDictError: PyException;
    GpuSupportError: PyException;
    InitInferenceRuntimeError: PyException;
    OpenZipFileError: PyException;
    ReadZipEntryError: PyException;
    ModelAlreadyLoadedError: PyException;
    StyleAlreadyLoadedError: PyException;
    InvalidModelFormatError: PyException;
    InvalidModelDataError: PyException;
    GetSupportedDevicesError: PyException;
    StyleNotFoundError: PyKeyError;
    ModelNotFoundError: PyKeyError;
    RunModelError: PyException;
    AnalyzeTextError: PyException;
    ParseKanaError: PyValueError;
    LoadUserDictError: PyException;
    SaveUserDictError: PyException;
    WordNotFoundError: PyKeyError;
    UseUserDictError: PyException;
    InvalidWordError: PyValueError;
}

struct Closable<T, C: PyTypeInfo, A: Async> {
    content: A::RwLock<MaybeClosed<T>>,
    marker: PhantomData<(C, A)>,
}

impl<T, C: PyTypeInfo, A: Async> Closable<T, C, A> {
    fn new(content: T) -> Self {
        Self {
            content: MaybeClosed::Open(content).into(),
            marker: PhantomData,
        }
    }

    fn read(&self) -> PyResult<impl Deref<Target = T> + '_> {
        let lock = self
            .content
            .try_read_()
            .map_err(|_| PyValueError::new_err(format!("The `{}` is being closed", C::NAME)))?;

        voicevox_core::__internal::interop::raii::try_map_guard(lock, |lock| match &**lock {
            MaybeClosed::Open(content) => Ok(content),
            MaybeClosed::Closed => Err(PyValueError::new_err(format!(
                "The `{}` is closed",
                C::NAME,
            ))),
        })
    }

    async fn close_(&self) -> Option<T> {
        let lock = &mut *match self.content.try_write_() {
            Ok(lock) => lock,
            Err(()) => {
                warn!("The `{}` is still in use. Waiting before closing", C::NAME);
                self.content.write_().await
            }
        };

        if matches!(*lock, MaybeClosed::Open(_)) {
            debug!("Closing a {}", C::NAME);
        }
        match mem::replace(lock, MaybeClosed::Closed) {
            MaybeClosed::Open(content) => Some(content),
            MaybeClosed::Closed => None,
        }
    }
}

impl<T, C: PyTypeInfo> Closable<T, C, SingleTasked> {
    #[must_use = "中身は明示的に`drop`でdropすること"]
    fn close(&self) -> Option<T> {
        futures_lite::future::block_on(self.close_())
    }
}

impl<T, C: PyTypeInfo> Closable<T, C, Tokio> {
    #[must_use = "中身は明示的に`drop`でdropすること"]
    async fn close(&self) -> Option<T> {
        self.close_().await
    }
}

trait Async {
    const EXIT_METHOD: &str;
    type RwLock<T>: RwLock<Item = T>;
}

enum SingleTasked {}
enum Tokio {}

impl Async for SingleTasked {
    const EXIT_METHOD: &str = "__exit__";
    type RwLock<T> = std::sync::RwLock<T>;
}

impl Async for Tokio {
    const EXIT_METHOD: &str = "__aexit__";
    type RwLock<T> = tokio::sync::RwLock<T>;
}

trait RwLock: From<Self::Item> {
    type Item;
    type RwLockWriteGuard<'a>: DerefMut<Target = Self::Item>
    where
        Self: 'a;
    fn try_read_(&self) -> Result<impl Deref<Target = Self::Item>, ()>;
    async fn write_(&self) -> Self::RwLockWriteGuard<'_>;
    fn try_write_(&self) -> Result<Self::RwLockWriteGuard<'_>, ()>;
}

impl<T> RwLock for std::sync::RwLock<T> {
    type Item = T;
    type RwLockWriteGuard<'a>
        = std::sync::RwLockWriteGuard<'a, Self::Item>
    where
        Self: 'a;

    fn try_read_(&self) -> Result<impl Deref<Target = Self::Item>, ()> {
        self.try_read().map_err(|e| match e {
            std::sync::TryLockError::Poisoned(e) => panic!("{e}"),
            std::sync::TryLockError::WouldBlock => (),
        })
    }

    async fn write_(&self) -> Self::RwLockWriteGuard<'_> {
        self.write().unwrap_or_else(|e| panic!("{e}"))
    }

    fn try_write_(&self) -> Result<Self::RwLockWriteGuard<'_>, ()> {
        self.try_write().map_err(|e| match e {
            std::sync::TryLockError::Poisoned(e) => panic!("{e}"),
            std::sync::TryLockError::WouldBlock => (),
        })
    }
}

impl<T> RwLock for tokio::sync::RwLock<T> {
    type Item = T;
    type RwLockWriteGuard<'a>
        = tokio::sync::RwLockWriteGuard<'a, Self::Item>
    where
        Self: 'a;

    fn try_read_(&self) -> Result<impl Deref<Target = Self::Item>, ()> {
        self.try_read().map_err(|_| ())
    }

    async fn write_(&self) -> Self::RwLockWriteGuard<'_> {
        self.write().await
    }

    fn try_write_(&self) -> Result<Self::RwLockWriteGuard<'_>, ()> {
        self.try_write().map_err(|_| ())
    }
}

#[derive(Clone)]
struct VoiceModelFilePyFields {
    id: PyObject,      // `NewType("VoiceModelId", UUID)`
    metas: Py<PyList>, // `list[CharacterMeta]`
}

#[pyfunction]
fn _validate_user_dict_word(ob: &PyAny) -> PyResult<()> {
    convert::to_rust_user_dict_word(ob).map(|_| ())
}

#[pyfunction]
fn _to_zenkaku(text: &str) -> PyResult<String> {
    Ok(voicevox_core::__internal::to_zenkaku(text))
}

#[pyfunction]
fn wav_from_s16le<'py>(
    pcm: &[u8],
    sampling_rate: u32,
    is_stereo: bool,
    py: Python<'py>,
) -> &'py PyBytes {
    PyBytes::new(
        py,
        &voicevox_core::__wav_from_s16le(pcm, sampling_rate, is_stereo),
    )
}

mod blocking {
    use std::{ffi::OsString, path::PathBuf, sync::Arc};

    use camino::Utf8PathBuf;
    use pyo3::{
        exceptions::{PyIndexError, PyTypeError, PyValueError},
        pyclass, pymethods,
        types::{IntoPyDict as _, PyBytes, PyDict, PyList, PyTuple, PyType},
        Py, PyAny, PyObject, PyRef, PyResult, Python,
    };
    use uuid::Uuid;
    use voicevox_core::{AccelerationMode, AudioQuery, StyleId, UserDictWord};

    use crate::{
        convert::{SupportedDevicesExt as _, VoicevoxCoreResultExt as _},
        Closable, SingleTasked, VoiceModelFilePyFields,
    };

    #[pyclass]
    #[derive(Clone)]
    pub(crate) struct VoiceModelFile {
        model: Arc<Closable<voicevox_core::blocking::VoiceModelFile, Self, SingleTasked>>,
        fields: VoiceModelFilePyFields,
    }

    #[pymethods]
    impl VoiceModelFile {
        #[new]
        #[classmethod]
        #[pyo3(signature = (*_args, **_kwargs))]
        fn new(_cls: &PyType, _args: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Self> {
            Err(PyTypeError::new_err((
                "`VoiceModelFile` does not have a normal constructor. Use \
                 `VoiceModelFile.load_once` to construct",
            )))
        }

        #[staticmethod]
        fn open(py: Python<'_>, path: PathBuf) -> PyResult<Self> {
            let model = voicevox_core::blocking::VoiceModelFile::open(path).into_py_result(py)?;

            let id = crate::convert::to_py_uuid(py, model.id().0)?;
            let metas = crate::convert::to_pydantic_voice_model_meta(model.metas(), py)?.into();

            let model = Closable::new(model).into();

            Ok(Self {
                model,
                fields: VoiceModelFilePyFields { id, metas },
            })
        }

        fn close(&self) {
            let this = self.model.close();
            drop(this);
        }

        #[getter]
        fn id(&self) -> PyObject {
            self.fields.id.clone()
        }

        #[getter]
        fn metas(&self) -> Py<PyList> {
            self.fields.metas.clone()
        }

        fn __enter__(slf: PyRef<'_, Self>) -> PyResult<PyRef<'_, Self>> {
            slf.model.read()?;
            Ok(slf)
        }

        fn __exit__(
            &self,
            #[expect(unused_variables, reason = "`__exit__`としては必要")] exc_type: &PyAny,
            #[expect(unused_variables, reason = "`__exit__`としては必要")] exc_value: &PyAny,
            #[expect(unused_variables, reason = "`__exit__`としては必要")] traceback: &PyAny,
        ) {
            self.close();
        }
    }

    static ONNXRUNTIME: once_cell::sync::OnceCell<Py<Onnxruntime>> =
        once_cell::sync::OnceCell::new();

    #[pyclass]
    #[derive(Clone)]
    pub(crate) struct Onnxruntime(&'static voicevox_core::blocking::Onnxruntime);

    #[pymethods]
    impl Onnxruntime {
        #[classattr]
        const LIB_NAME: &'static str = voicevox_core::blocking::Onnxruntime::LIB_NAME;

        #[classattr]
        const LIB_VERSION: &'static str = voicevox_core::blocking::Onnxruntime::LIB_VERSION;

        #[classattr]
        const LIB_VERSIONED_FILENAME: &'static str =
            voicevox_core::blocking::Onnxruntime::LIB_VERSIONED_FILENAME;

        #[classattr]
        const LIB_UNVERSIONED_FILENAME: &'static str =
            voicevox_core::blocking::Onnxruntime::LIB_UNVERSIONED_FILENAME;

        #[new]
        #[classmethod]
        #[pyo3(signature = (*_args, **_kwargs))]
        fn new(_cls: &PyType, _args: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Self> {
            Err(PyTypeError::new_err((
                "`Onnxruntime` does not have a normal constructor. Use `Onnxruntime.load_once` or \
                 `Onnxruntime.get` to construct",
            )))
        }

        #[staticmethod]
        fn get(py: Python<'_>) -> PyResult<Option<Py<Self>>> {
            let result = ONNXRUNTIME.get_or_try_init(|| {
                match voicevox_core::blocking::Onnxruntime::get().map(|o| Py::new(py, Self(o))) {
                    Some(Ok(this)) => Ok(this),
                    Some(Err(err)) => Err(Some(err)),
                    None => Err(None),
                }
            });

            match result {
                Ok(this) => Ok(Some(this.clone())),
                Err(Some(err)) => Err(err),
                Err(None) => Ok(None),
            }
        }

        #[staticmethod]
        #[pyo3(signature = (*, filename = Self::LIB_VERSIONED_FILENAME.into()))]
        fn load_once(filename: OsString, py: Python<'_>) -> PyResult<Py<Self>> {
            ONNXRUNTIME
                .get_or_try_init(|| {
                    let inner = voicevox_core::blocking::Onnxruntime::load_once()
                        .filename(filename)
                        .perform()
                        .into_py_result(py)?;
                    Py::new(py, Self(inner))
                })
                .cloned()
        }

        fn supported_devices<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
            self.0.supported_devices().into_py_result(py)?.to_py(py)
        }
    }

    #[pyclass]
    #[derive(Clone)]
    pub(crate) struct OpenJtalk {
        open_jtalk: voicevox_core::blocking::OpenJtalk,
    }

    #[pymethods]
    impl OpenJtalk {
        #[new]
        fn new(
            #[pyo3(from_py_with = "super::from_utf8_path")] open_jtalk_dict_dir: Utf8PathBuf,
            py: Python<'_>,
        ) -> PyResult<Self> {
            let open_jtalk =
                voicevox_core::blocking::OpenJtalk::new(open_jtalk_dict_dir).into_py_result(py)?;
            Ok(Self { open_jtalk })
        }

        fn use_user_dict(&self, user_dict: UserDict, py: Python<'_>) -> PyResult<()> {
            self.open_jtalk
                .use_user_dict(&user_dict.dict)
                .into_py_result(py)
        }
    }

    #[pyclass]
    pub(crate) struct AudioFeature {
        audio: voicevox_core::blocking::__AudioFeature,
    }

    #[pymethods]
    impl AudioFeature {
        #[getter]
        fn frame_length(&self) -> usize {
            self.audio.frame_length
        }

        #[getter]
        fn frame_rate(&self) -> f64 {
            self.audio.frame_rate
        }
    }

    #[pyclass]
    pub(crate) struct Synthesizer {
        synthesizer: Closable<
            voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>,
            Self,
            SingleTasked,
        >,
    }

    #[pymethods]
    impl Synthesizer {
        #[new]
        #[pyo3(signature =(
            onnxruntime,
            open_jtalk,
            *,
            acceleration_mode = Default::default(),
            cpu_num_threads = voicevox_core::__internal::interop::DEFAULT_CPU_NUM_THREADS,
        ))]
        fn new(
            onnxruntime: Onnxruntime,
            open_jtalk: OpenJtalk,
            #[pyo3(from_py_with = "crate::convert::from_acceleration_mode")]
            acceleration_mode: AccelerationMode,
            cpu_num_threads: u16,
            py: Python<'_>,
        ) -> PyResult<Self> {
            let inner = voicevox_core::blocking::Synthesizer::builder(onnxruntime.0)
                .text_analyzer(open_jtalk.open_jtalk.clone())
                .acceleration_mode(acceleration_mode)
                .cpu_num_threads(cpu_num_threads)
                .build()
                .into_py_result(py)?;
            Ok(Self {
                synthesizer: Closable::new(inner),
            })
        }

        fn __repr__(&self) -> &'static str {
            "Synthesizer { .. }"
        }

        fn __enter__(slf: PyRef<'_, Self>) -> PyResult<PyRef<'_, Self>> {
            slf.synthesizer.read()?;
            Ok(slf)
        }

        fn __exit__(
            &mut self,
            #[expect(unused_variables, reason = "`__exit__`としては必要")] exc_type: &PyAny,
            #[expect(unused_variables, reason = "`__exit__`としては必要")] exc_value: &PyAny,
            #[expect(unused_variables, reason = "`__exit__`としては必要")] traceback: &PyAny,
        ) {
            self.close();
        }

        #[getter]
        fn onnxruntime(&self) -> Py<Onnxruntime> {
            ONNXRUNTIME.get().expect("should be initialized").clone()
        }

        #[getter]
        fn is_gpu_mode(&self) -> PyResult<bool> {
            let synthesizer = self.synthesizer.read()?;
            Ok(synthesizer.is_gpu_mode())
        }

        fn metas<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
            let synthesizer = self.synthesizer.read()?;
            crate::convert::to_pydantic_voice_model_meta(&synthesizer.metas(), py)
        }

        fn load_voice_model(&mut self, model: &PyAny, py: Python<'_>) -> PyResult<()> {
            let this = self.synthesizer.read()?;
            let model = model.extract::<VoiceModelFile>()?;
            let model = &model.model.read()?;
            this.load_voice_model(model).into_py_result(py)
        }

        fn unload_voice_model(
            &mut self,
            #[pyo3(from_py_with = "crate::convert::to_rust_uuid")] voice_model_id: Uuid,
            py: Python<'_>,
        ) -> PyResult<()> {
            self.synthesizer
                .read()?
                .unload_voice_model(voice_model_id.into())
                .into_py_result(py)
        }

        fn is_loaded_voice_model(
            &self,
            #[pyo3(from_py_with = "crate::convert::to_rust_uuid")] voice_model_id: Uuid,
        ) -> PyResult<bool> {
            Ok(self
                .synthesizer
                .read()?
                .is_loaded_voice_model(voice_model_id.into()))
        }

        fn create_audio_query_from_kana<'py>(
            &self,
            kana: &str,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.read()?;

            let audio_query = synthesizer
                .create_audio_query_from_kana(kana, StyleId::new(style_id))
                .into_py_result(py)?;

            let class = py.import("voicevox_core")?.getattr("AudioQuery")?;
            crate::convert::to_pydantic_dataclass(audio_query, class)
        }

        fn create_audio_query<'py>(
            &self,
            text: &str,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizesr = self.synthesizer.read()?;

            let audio_query = synthesizesr
                .create_audio_query(text, StyleId::new(style_id))
                .into_py_result(py)?;

            let class = py.import("voicevox_core")?.getattr("AudioQuery")?;
            crate::convert::to_pydantic_dataclass(audio_query, class)
        }

        fn create_accent_phrases_from_kana<'py>(
            &self,
            kana: &str,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<Vec<&'py PyAny>> {
            let synthesizer = self.synthesizer.read()?;

            let accent_phrases = synthesizer
                .create_accent_phrases_from_kana(kana, StyleId::new(style_id))
                .into_py_result(py)?;

            let class = py.import("voicevox_core")?.getattr("AccentPhrase")?;
            accent_phrases
                .iter()
                .map(|ap| crate::convert::to_pydantic_dataclass(ap, class))
                .collect()
        }

        fn create_accent_phrases<'py>(
            &self,
            text: &str,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<Vec<&'py PyAny>> {
            let synthesizer = self.synthesizer.read()?;

            let accent_phrases = synthesizer
                .create_accent_phrases(text, StyleId::new(style_id))
                .into_py_result(py)?;

            let class = py.import("voicevox_core")?.getattr("AccentPhrase")?;
            accent_phrases
                .iter()
                .map(|ap| crate::convert::to_pydantic_dataclass(ap, class))
                .collect()
        }

        fn replace_mora_data<'py>(
            &self,
            accent_phrases: &'py PyList,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<Vec<&'py PyAny>> {
            let synthesizer = self.synthesizer.read()?;
            crate::convert::blocking_modify_accent_phrases(
                accent_phrases,
                StyleId::new(style_id),
                py,
                |a, s| synthesizer.replace_mora_data(&a, s),
            )
        }

        fn replace_phoneme_length<'py>(
            &self,
            accent_phrases: &'py PyList,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<Vec<&'py PyAny>> {
            let synthesizer = self.synthesizer.read()?;
            crate::convert::blocking_modify_accent_phrases(
                accent_phrases,
                StyleId::new(style_id),
                py,
                |a, s| synthesizer.replace_phoneme_length(&a, s),
            )
        }

        fn replace_mora_pitch<'py>(
            &self,
            accent_phrases: &'py PyList,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<Vec<&'py PyAny>> {
            let synthesizer = self.synthesizer.read()?;
            crate::convert::blocking_modify_accent_phrases(
                accent_phrases,
                StyleId::new(style_id),
                py,
                |a, s| synthesizer.replace_mora_pitch(&a, s),
            )
        }

        // TODO: 後で復活させる
        // https://github.com/VOICEVOX/voicevox_core/issues/970
        #[allow(non_snake_case)]
        #[pyo3(signature=(
            audio_query,
            style_id,
            *,
            enable_interrogative_upspeak =
                voicevox_core::__internal::interop::DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
        ))]
        fn _Synthesizer__precompute_render(
            &self,
            #[pyo3(from_py_with = "crate::convert::from_dataclass")] audio_query: AudioQuery,
            style_id: u32,
            enable_interrogative_upspeak: bool,
            py: Python<'_>,
        ) -> PyResult<AudioFeature> {
            let audio = self
                .synthesizer
                .read()?
                .__precompute_render(&audio_query, StyleId::new(style_id))
                .enable_interrogative_upspeak(enable_interrogative_upspeak)
                .perform()
                .into_py_result(py)?;
            Ok(AudioFeature { audio })
        }

        // TODO: 後で復活させる
        // https://github.com/VOICEVOX/voicevox_core/issues/970
        #[allow(non_snake_case)]
        fn _Synthesizer__render<'py>(
            &self,
            audio: &AudioFeature,
            start: usize,
            stop: usize,
            py: Python<'py>,
        ) -> PyResult<&'py PyBytes> {
            if start > audio.frame_length() || stop > audio.frame_length() {
                return Err(PyIndexError::new_err(format!(
                    "({start}, {stop}) is out of range for audio feature of length {len}",
                    len = audio.frame_length(),
                )));
            }
            if start > stop {
                return Err(PyValueError::new_err(format!(
                    "({start}, {stop}) is invalid range because start > end",
                )));
            }
            let wav = &self
                .synthesizer
                .read()?
                .__render(&audio.audio, start..stop)
                .into_py_result(py)?;
            Ok(PyBytes::new(py, wav))
        }

        #[pyo3(signature=(
            audio_query,
            style_id,
            *,
            enable_interrogative_upspeak =
                voicevox_core::__internal::interop::DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
        ))]
        fn synthesis<'py>(
            &self,
            #[pyo3(from_py_with = "crate::convert::from_dataclass")] audio_query: AudioQuery,
            style_id: u32,
            enable_interrogative_upspeak: bool,
            py: Python<'py>,
        ) -> PyResult<&'py PyBytes> {
            let wav = &self
                .synthesizer
                .read()?
                .synthesis(&audio_query, StyleId::new(style_id))
                .enable_interrogative_upspeak(enable_interrogative_upspeak)
                .perform()
                .into_py_result(py)?;
            Ok(PyBytes::new(py, wav))
        }

        #[pyo3(signature=(
            kana,
            style_id,
            *,
            enable_interrogative_upspeak =
                voicevox_core::__internal::interop::DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
        ))]
        fn tts_from_kana<'py>(
            &self,
            kana: &str,
            style_id: u32,
            enable_interrogative_upspeak: bool,
            py: Python<'py>,
        ) -> PyResult<&'py PyBytes> {
            let style_id = StyleId::new(style_id);
            let wav = &self
                .synthesizer
                .read()?
                .tts_from_kana(kana, style_id)
                .enable_interrogative_upspeak(enable_interrogative_upspeak)
                .perform()
                .into_py_result(py)?;
            Ok(PyBytes::new(py, wav))
        }

        #[pyo3(signature=(
            text,
            style_id,
            *,
            enable_interrogative_upspeak =
                voicevox_core::__internal::interop::DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
        ))]
        fn tts<'py>(
            &self,
            text: &str,
            style_id: u32,
            enable_interrogative_upspeak: bool,
            py: Python<'py>,
        ) -> PyResult<&'py PyBytes> {
            let style_id = StyleId::new(style_id);
            let wav = &self
                .synthesizer
                .read()?
                .tts(text, style_id)
                .enable_interrogative_upspeak(enable_interrogative_upspeak)
                .perform()
                .into_py_result(py)?;
            Ok(PyBytes::new(py, wav))
        }

        fn close(&mut self) {
            drop(self.synthesizer.close());
        }
    }

    #[pyclass]
    #[derive(Default, Debug, Clone)]
    pub(crate) struct UserDict {
        dict: Arc<voicevox_core::blocking::UserDict>,
    }

    #[pymethods]
    impl UserDict {
        #[new]
        fn new() -> Self {
            Self::default()
        }

        fn load(&self, path: PathBuf, py: Python<'_>) -> PyResult<()> {
            self.dict.load(path).into_py_result(py)
        }

        fn save(&self, path: PathBuf, py: Python<'_>) -> PyResult<()> {
            self.dict.save(path).into_py_result(py)
        }

        fn add_word(
            &mut self,
            #[pyo3(from_py_with = "crate::convert::to_rust_user_dict_word")] word: UserDictWord,
            py: Python<'_>,
        ) -> PyResult<PyObject> {
            let uuid = self.dict.add_word(word).into_py_result(py)?;

            crate::convert::to_py_uuid(py, uuid)
        }

        fn update_word(
            &mut self,
            #[pyo3(from_py_with = "crate::convert::to_rust_uuid")] word_uuid: Uuid,
            #[pyo3(from_py_with = "crate::convert::to_rust_user_dict_word")] word: UserDictWord,
            py: Python<'_>,
        ) -> PyResult<()> {
            self.dict.update_word(word_uuid, word).into_py_result(py)
        }

        fn remove_word(
            &mut self,
            #[pyo3(from_py_with = "crate::convert::to_rust_uuid")] word_uuid: Uuid,
            py: Python<'_>,
        ) -> PyResult<()> {
            self.dict.remove_word(word_uuid).into_py_result(py)?;
            Ok(())
        }

        fn import_dict(&mut self, other: &UserDict, py: Python<'_>) -> PyResult<()> {
            self.dict.import(&other.dict).into_py_result(py)?;
            Ok(())
        }

        fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
            let words = self.dict.with_words(|words| {
                words
                    .iter()
                    .map(|(&uuid, word)| {
                        let uuid = crate::convert::to_py_uuid(py, uuid)?;
                        let word = crate::convert::to_py_user_dict_word(py, word)?;
                        Ok((uuid, word))
                    })
                    .collect::<PyResult<Vec<_>>>()
            })?;
            Ok(words.into_py_dict(py))
        }
    }
}

mod asyncio {
    use std::{ffi::OsString, path::PathBuf, sync::Arc};

    use camino::Utf8PathBuf;
    use pyo3::{
        exceptions::PyTypeError,
        pyclass, pymethods,
        types::{IntoPyDict as _, PyBytes, PyDict, PyList, PyTuple, PyType},
        Py, PyAny, PyErr, PyObject, PyRef, PyResult, Python, ToPyObject as _,
    };
    use uuid::Uuid;
    use voicevox_core::{AccelerationMode, AudioQuery, StyleId, UserDictWord};

    use crate::{
        convert::{SupportedDevicesExt as _, VoicevoxCoreResultExt as _},
        Closable, Tokio, VoiceModelFilePyFields,
    };

    #[pyclass]
    #[derive(Clone)]
    pub(crate) struct VoiceModelFile {
        model: Arc<Closable<voicevox_core::nonblocking::VoiceModelFile, Self, Tokio>>,
        fields: VoiceModelFilePyFields,
    }

    #[pymethods]
    impl VoiceModelFile {
        #[new]
        #[classmethod]
        #[pyo3(signature = (*_args, **_kwargs))]
        fn new(_cls: &PyType, _args: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Self> {
            Err(PyTypeError::new_err((
                "`VoiceModelFile` does not have a normal constructor. Use \
                 `VoiceModelFile.load_once` to construct",
            )))
        }

        #[staticmethod]
        fn open(py: Python<'_>, path: PathBuf) -> PyResult<&PyAny> {
            pyo3_asyncio::tokio::future_into_py(py, async move {
                let model = voicevox_core::nonblocking::VoiceModelFile::open(path).await;
                let (model, id, metas) = Python::with_gil(|py| {
                    let model = Python::with_gil(|py| model.into_py_result(py))?;
                    let id = crate::convert::to_py_uuid(py, model.id().0)?;
                    let metas =
                        crate::convert::to_pydantic_voice_model_meta(model.metas(), py)?.into();
                    Ok::<_, PyErr>((model, id, metas))
                })?;

                let model = Closable::new(model).into();

                Ok(Self {
                    model,
                    fields: VoiceModelFilePyFields { id, metas },
                })
            })
        }

        fn close<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
            let this = self.model.clone();
            pyo3_asyncio::tokio::future_into_py(py, async move {
                if let Some(this) = this.close().await {
                    this.close().await;
                }
                Ok(())
            })
        }

        #[getter]
        fn id(&self) -> PyObject {
            self.fields.id.clone()
        }

        #[getter]
        fn metas(&self) -> Py<PyList> {
            self.fields.metas.clone()
        }

        fn __aenter__(slf: PyRef<'_, Self>) -> PyResult<&PyAny> {
            slf.model.read()?;

            let py = slf.py();
            crate::convert::ready(slf, py)
        }

        fn __aexit__<'py>(
            &self,
            #[expect(unused_variables, reason = "`__aexit__`としては必要")] exc_type: &'py PyAny,
            #[expect(unused_variables, reason = "`__aexit__`としては必要")] exc_value: &'py PyAny,
            #[expect(unused_variables, reason = "`__aexit__`としては必要")] traceback: &'py PyAny,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            self.close(py)
        }
    }

    static ONNXRUNTIME: once_cell::sync::OnceCell<Py<Onnxruntime>> =
        once_cell::sync::OnceCell::new();

    #[pyclass]
    #[derive(Clone)]
    pub(crate) struct Onnxruntime(&'static voicevox_core::nonblocking::Onnxruntime);

    #[pymethods]
    impl Onnxruntime {
        #[classattr]
        const LIB_NAME: &'static str = voicevox_core::nonblocking::Onnxruntime::LIB_NAME;

        #[classattr]
        const LIB_VERSION: &'static str = voicevox_core::nonblocking::Onnxruntime::LIB_VERSION;

        #[classattr]
        const LIB_VERSIONED_FILENAME: &'static str =
            voicevox_core::nonblocking::Onnxruntime::LIB_VERSIONED_FILENAME;

        #[classattr]
        const LIB_UNVERSIONED_FILENAME: &'static str =
            voicevox_core::nonblocking::Onnxruntime::LIB_UNVERSIONED_FILENAME;

        #[new]
        #[classmethod]
        #[pyo3(signature = (*_args, **_kwargs))]
        fn new(_cls: &PyType, _args: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Self> {
            Err(PyTypeError::new_err((
                "`Onnxruntime` does not have a normal constructor. Use `Onnxruntime.load_once` or \
                 `Onnxruntime.get` to construct",
            )))
        }

        #[staticmethod]
        fn get(py: Python<'_>) -> PyResult<Option<Py<Self>>> {
            let result =
                ONNXRUNTIME.get_or_try_init(
                    || match voicevox_core::nonblocking::Onnxruntime::get()
                        .map(|o| Py::new(py, Self(o)))
                    {
                        Some(Ok(this)) => Ok(this),
                        Some(Err(err)) => Err(Some(err)),
                        None => Err(None),
                    },
                );

            match result {
                Ok(this) => Ok(Some(this.clone())),
                Err(Some(err)) => Err(err),
                Err(None) => Ok(None),
            }
        }

        #[staticmethod]
        #[pyo3(signature = (*, filename = Self::LIB_VERSIONED_FILENAME.into()))]
        fn load_once(filename: OsString, py: Python<'_>) -> PyResult<&PyAny> {
            pyo3_asyncio::tokio::future_into_py(py, async move {
                let inner = voicevox_core::nonblocking::Onnxruntime::load_once()
                    .filename(filename)
                    .perform()
                    .await;

                ONNXRUNTIME.get_or_try_init(|| {
                    Python::with_gil(|py| Py::new(py, Self(inner.into_py_result(py)?)))
                })
            })
        }

        fn supported_devices<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
            self.0.supported_devices().into_py_result(py)?.to_py(py)
        }
    }

    #[pyclass]
    #[derive(Clone)]
    pub(crate) struct OpenJtalk {
        open_jtalk: voicevox_core::nonblocking::OpenJtalk,
    }

    #[pymethods]
    impl OpenJtalk {
        #[new]
        #[classmethod]
        #[pyo3(signature = (*_args, **_kwargs))]
        fn __new__(_cls: &PyType, _args: &PyTuple, _kwargs: Option<&PyDict>) -> PyResult<Self> {
            Err(PyTypeError::new_err((
                "`OpenJtalk` does not have a normal constructor. Use `OpenJtalk.new` to construct",
            )))
        }

        #[expect(clippy::new_ret_no_self, reason = "これはPython API")]
        #[staticmethod]
        fn new(
            #[pyo3(from_py_with = "crate::convert::from_utf8_path")]
            open_jtalk_dict_dir: Utf8PathBuf,
            py: Python<'_>,
        ) -> PyResult<&PyAny> {
            pyo3_asyncio::tokio::future_into_py(py, async move {
                let open_jtalk =
                    voicevox_core::nonblocking::OpenJtalk::new(open_jtalk_dict_dir).await;
                let open_jtalk = Python::with_gil(|py| open_jtalk.into_py_result(py))?;
                Ok(Self { open_jtalk })
            })
        }

        fn use_user_dict<'py>(&self, user_dict: UserDict, py: Python<'py>) -> PyResult<&'py PyAny> {
            let this = self.open_jtalk.clone();

            pyo3_asyncio::tokio::future_into_py(py, async move {
                let result = this.use_user_dict(&user_dict.dict).await;
                Python::with_gil(|py| result.into_py_result(py))
            })
        }
    }

    #[pyclass]
    pub(crate) struct Synthesizer {
        synthesizer: Arc<
            Closable<
                voicevox_core::nonblocking::Synthesizer<voicevox_core::nonblocking::OpenJtalk>,
                Self,
                Tokio,
            >,
        >,
    }

    #[pymethods]
    impl Synthesizer {
        #[new]
        #[pyo3(signature =(
            onnxruntime,
            open_jtalk,
            *,
            acceleration_mode = Default::default(),
            cpu_num_threads = voicevox_core::__internal::interop::DEFAULT_CPU_NUM_THREADS,
        ))]
        fn new(
            onnxruntime: Onnxruntime,
            open_jtalk: OpenJtalk,
            #[pyo3(from_py_with = "crate::convert::from_acceleration_mode")]
            acceleration_mode: AccelerationMode,
            cpu_num_threads: u16,
        ) -> PyResult<Self> {
            let synthesizer = voicevox_core::nonblocking::Synthesizer::builder(onnxruntime.0)
                .text_analyzer(open_jtalk.open_jtalk.clone())
                .acceleration_mode(acceleration_mode)
                .cpu_num_threads(cpu_num_threads)
                .build();
            let synthesizer = Python::with_gil(|py| synthesizer.into_py_result(py))?;
            let synthesizer = Closable::new(synthesizer).into();
            Ok(Self { synthesizer })
        }

        fn __repr__(&self) -> &'static str {
            "Synthesizer { .. }"
        }

        fn __aenter__(slf: PyRef<'_, Self>) -> PyResult<&PyAny> {
            slf.synthesizer.read()?;

            let py = slf.py();
            crate::convert::ready(slf, py)
        }

        fn __aexit__<'py>(
            &mut self,
            #[expect(unused_variables, reason = "`__aexit__`としては必要")] exc_type: &'py PyAny,
            #[expect(unused_variables, reason = "`__aexit__`としては必要")] exc_value: &'py PyAny,
            #[expect(unused_variables, reason = "`__aexit__`としては必要")] traceback: &'py PyAny,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            self.close(py)
        }

        #[getter]
        fn onnxruntime(&self) -> Py<Onnxruntime> {
            ONNXRUNTIME.get().expect("should be initialized").clone()
        }

        #[getter]
        fn is_gpu_mode(&self) -> PyResult<bool> {
            let synthesizer = self.synthesizer.read()?;
            Ok(synthesizer.is_gpu_mode())
        }

        fn metas<'py>(&self, py: Python<'py>) -> PyResult<&'py PyList> {
            let synthesizer = self.synthesizer.read()?;
            crate::convert::to_pydantic_voice_model_meta(&synthesizer.metas(), py)
        }

        fn load_voice_model<'py>(
            &mut self,
            model: &'py PyAny,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let model: VoiceModelFile = model.extract()?;
            let synthesizer = self.synthesizer.clone();
            pyo3_asyncio::tokio::future_into_py(py, async move {
                let result = synthesizer
                    .read()?
                    .load_voice_model(&*model.model.read()?)
                    .await;
                Python::with_gil(|py| result.into_py_result(py))
            })
        }

        fn unload_voice_model(
            &mut self,
            #[pyo3(from_py_with = "crate::convert::to_rust_uuid")] voice_model_id: Uuid,
            py: Python<'_>,
        ) -> PyResult<()> {
            self.synthesizer
                .read()?
                .unload_voice_model(voice_model_id.into())
                .into_py_result(py)
        }

        fn is_loaded_voice_model(
            &self,
            #[pyo3(from_py_with = "crate::convert::to_rust_uuid")] voice_model_id: Uuid,
        ) -> PyResult<bool> {
            Ok(self
                .synthesizer
                .read()?
                .is_loaded_voice_model(voice_model_id.into()))
        }

        fn create_audio_query_from_kana<'py>(
            &self,
            kana: &str,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.clone();
            let kana = kana.to_owned();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let audio_query = synthesizer
                        .read()?
                        .create_audio_query_from_kana(&kana, StyleId::new(style_id))
                        .await;

                    Python::with_gil(|py| {
                        let class = py.import("voicevox_core")?.getattr("AudioQuery")?;
                        let ret = crate::convert::to_pydantic_dataclass(
                            audio_query.into_py_result(py)?,
                            class,
                        )?;
                        Ok(ret.to_object(py))
                    })
                },
            )
        }

        fn create_audio_query<'py>(
            &self,
            text: &str,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.clone();
            let text = text.to_owned();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let audio_query = synthesizer
                        .read()?
                        .create_audio_query(&text, StyleId::new(style_id))
                        .await;

                    Python::with_gil(|py| {
                        let audio_query = audio_query.into_py_result(py)?;
                        let class = py.import("voicevox_core")?.getattr("AudioQuery")?;
                        let ret = crate::convert::to_pydantic_dataclass(audio_query, class)?;
                        Ok(ret.to_object(py))
                    })
                },
            )
        }

        fn create_accent_phrases_from_kana<'py>(
            &self,
            kana: &str,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.clone();
            let kana = kana.to_owned();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let accent_phrases = synthesizer
                        .read()?
                        .create_accent_phrases_from_kana(&kana, StyleId::new(style_id))
                        .await;
                    Python::with_gil(|py| {
                        let class = py.import("voicevox_core")?.getattr("AccentPhrase")?;
                        let accent_phrases = accent_phrases
                            .into_py_result(py)?
                            .iter()
                            .map(|ap| crate::convert::to_pydantic_dataclass(ap, class))
                            .collect::<PyResult<Vec<_>>>();
                        let list = PyList::new(py, accent_phrases);
                        Ok(list.to_object(py))
                    })
                },
            )
        }

        fn create_accent_phrases<'py>(
            &self,
            text: &str,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.clone();
            let text = text.to_owned();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let accent_phrases = synthesizer
                        .read()?
                        .create_accent_phrases(&text, StyleId::new(style_id))
                        .await;
                    Python::with_gil(|py| {
                        let class = py.import("voicevox_core")?.getattr("AccentPhrase")?;
                        let accent_phrases = accent_phrases
                            .into_py_result(py)?
                            .iter()
                            .map(|ap| crate::convert::to_pydantic_dataclass(ap, class))
                            .collect::<PyResult<Vec<_>>>();
                        let list = PyList::new(py, accent_phrases);
                        Ok(list.to_object(py))
                    })
                },
            )
        }

        fn replace_mora_data<'py>(
            &self,
            accent_phrases: &'py PyList,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.clone();
            crate::convert::async_modify_accent_phrases(
                accent_phrases,
                StyleId::new(style_id),
                py,
                |a, s| async move {
                    let result = synthesizer.read()?.replace_mora_data(&a, s).await;
                    Python::with_gil(|py| result.into_py_result(py))
                },
            )
        }

        fn replace_phoneme_length<'py>(
            &self,
            accent_phrases: &'py PyList,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.clone();
            crate::convert::async_modify_accent_phrases(
                accent_phrases,
                StyleId::new(style_id),
                py,
                |a, s| async move {
                    let result = synthesizer.read()?.replace_phoneme_length(&a, s).await;
                    Python::with_gil(|py| result.into_py_result(py))
                },
            )
        }

        fn replace_mora_pitch<'py>(
            &self,
            accent_phrases: &'py PyList,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.clone();
            crate::convert::async_modify_accent_phrases(
                accent_phrases,
                StyleId::new(style_id),
                py,
                |a, s| async move {
                    let result = synthesizer.read()?.replace_mora_pitch(&a, s).await;
                    Python::with_gil(|py| result.into_py_result(py))
                },
            )
        }

        #[pyo3(signature=(
            audio_query,
            style_id,
            *,
            enable_interrogative_upspeak =
                voicevox_core::__internal::interop::DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
        ))]
        fn synthesis<'py>(
            &self,
            #[pyo3(from_py_with = "crate::convert::from_dataclass")] audio_query: AudioQuery,
            style_id: u32,
            enable_interrogative_upspeak: bool,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.clone();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let wav = synthesizer
                        .read()?
                        .synthesis(&audio_query, StyleId::new(style_id))
                        .enable_interrogative_upspeak(enable_interrogative_upspeak)
                        .perform()
                        .await;
                    Python::with_gil(|py| {
                        let wav = wav.into_py_result(py)?;
                        Ok(PyBytes::new(py, &wav).to_object(py))
                    })
                },
            )
        }

        #[pyo3(signature=(
            kana,
            style_id,
            *,
            enable_interrogative_upspeak =
                voicevox_core::__internal::interop::DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
        ))]
        fn tts_from_kana<'py>(
            &self,
            kana: &str,
            style_id: u32,
            enable_interrogative_upspeak: bool,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let style_id = StyleId::new(style_id);
            let synthesizer = self.synthesizer.clone();
            let kana = kana.to_owned();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let wav = synthesizer
                        .read()?
                        .tts_from_kana(&kana, style_id)
                        .enable_interrogative_upspeak(enable_interrogative_upspeak)
                        .perform()
                        .await;

                    Python::with_gil(|py| {
                        let wav = wav.into_py_result(py)?;
                        Ok(PyBytes::new(py, &wav).to_object(py))
                    })
                },
            )
        }

        #[pyo3(signature=(
            text,
            style_id,
            *,
            enable_interrogative_upspeak =
                voicevox_core::__internal::interop::DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
        ))]
        fn tts<'py>(
            &self,
            text: &str,
            style_id: u32,
            enable_interrogative_upspeak: bool,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let style_id = StyleId::new(style_id);
            let synthesizer = self.synthesizer.clone();
            let text = text.to_owned();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let wav = synthesizer
                        .read()?
                        .tts(&text, style_id)
                        .enable_interrogative_upspeak(enable_interrogative_upspeak)
                        .perform()
                        .await;

                    Python::with_gil(|py| {
                        let wav = wav.into_py_result(py)?;
                        Ok(PyBytes::new(py, &wav).to_object(py))
                    })
                },
            )
        }

        fn close<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
            let this = self.synthesizer.clone();
            pyo3_asyncio::tokio::future_into_py(py, async move {
                if let Some(this) = this.close().await {
                    crate::convert::run_in_executor(|| drop(this)).await?;
                }
                Ok(())
            })
        }
    }

    #[pyclass]
    #[derive(Default, Debug, Clone)]
    pub(crate) struct UserDict {
        dict: Arc<voicevox_core::nonblocking::UserDict>,
    }

    #[pymethods]
    impl UserDict {
        #[new]
        fn new() -> Self {
            Self::default()
        }

        fn load<'py>(&self, path: PathBuf, py: Python<'py>) -> PyResult<&'py PyAny> {
            let this = self.dict.clone();

            pyo3_asyncio::tokio::future_into_py(py, async move {
                let result = this.load(&path).await;
                Python::with_gil(|py| result.into_py_result(py))
            })
        }

        fn save<'py>(&self, path: PathBuf, py: Python<'py>) -> PyResult<&'py PyAny> {
            let this = self.dict.clone();

            pyo3_asyncio::tokio::future_into_py(py, async move {
                let result = this.save(&path).await;
                Python::with_gil(|py| result.into_py_result(py))
            })
        }

        fn add_word(
            &mut self,
            #[pyo3(from_py_with = "crate::convert::to_rust_user_dict_word")] word: UserDictWord,
            py: Python<'_>,
        ) -> PyResult<PyObject> {
            let uuid = self.dict.add_word(word).into_py_result(py)?;

            crate::convert::to_py_uuid(py, uuid)
        }

        fn update_word(
            &mut self,
            #[pyo3(from_py_with = "crate::convert::to_rust_uuid")] word_uuid: Uuid,
            #[pyo3(from_py_with = "crate::convert::to_rust_user_dict_word")] word: UserDictWord,
            py: Python<'_>,
        ) -> PyResult<()> {
            self.dict.update_word(word_uuid, word).into_py_result(py)?;
            Ok(())
        }

        fn remove_word(
            &mut self,
            #[pyo3(from_py_with = "crate::convert::to_rust_uuid")] word_uuid: Uuid,
            py: Python<'_>,
        ) -> PyResult<()> {
            self.dict.remove_word(word_uuid).into_py_result(py)?;
            Ok(())
        }

        fn import_dict(&mut self, other: &UserDict, py: Python<'_>) -> PyResult<()> {
            self.dict.import(&other.dict).into_py_result(py)?;
            Ok(())
        }

        fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
            let words = self.dict.with_words(|words| {
                words
                    .iter()
                    .map(|(&uuid, word)| {
                        let uuid = crate::convert::to_py_uuid(py, uuid)?;
                        let word = crate::convert::to_py_user_dict_word(py, word)?;
                        Ok((uuid, word))
                    })
                    .collect::<PyResult<Vec<_>>>()
            })?;
            Ok(words.into_py_dict(py))
        }
    }
}
