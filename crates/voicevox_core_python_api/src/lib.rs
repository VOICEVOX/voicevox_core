use std::marker::PhantomData;

mod convert;
use self::convert::{from_utf8_path, VoicevoxCoreResultExt as _};
use easy_ext::ext;
use log::debug;
use pyo3::{
    create_exception,
    exceptions::{PyException, PyKeyError, PyValueError},
    pyfunction, pymodule,
    types::PyModule,
    wrap_pyfunction, PyResult, PyTypeInfo, Python,
};

#[pymodule]
#[pyo3(name = "_rust")]
fn rust(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_wrapped(wrap_pyfunction!(_validate_pronunciation))?;
    module.add_wrapped(wrap_pyfunction!(_to_zenkaku))?;

    add_exceptions(module)?;

    let blocking_module = PyModule::new(py, "voicevox_core._rust.blocking")?;
    blocking_module.add_class::<self::blocking::Synthesizer>()?;
    blocking_module.add_class::<self::blocking::Onnxruntime>()?;
    blocking_module.add_class::<self::blocking::OpenJtalk>()?;
    blocking_module.add_class::<self::blocking::VoiceModel>()?;
    blocking_module.add_class::<self::blocking::UserDict>()?;
    module.add_and_register_submodule(blocking_module)?;

    let asyncio_module = PyModule::new(py, "voicevox_core._rust.asyncio")?;
    asyncio_module.add_class::<self::asyncio::Synthesizer>()?;
    asyncio_module.add_class::<self::asyncio::Onnxruntime>()?;
    asyncio_module.add_class::<self::asyncio::OpenJtalk>()?;
    asyncio_module.add_class::<self::asyncio::VoiceModel>()?;
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
    ExtractFullContextLabelError: PyException;
    ParseKanaError: PyValueError;
    LoadUserDictError: PyException;
    SaveUserDictError: PyException;
    WordNotFoundError: PyKeyError;
    UseUserDictError: PyException;
    InvalidWordError: PyValueError;
}

struct Closable<T, C: PyTypeInfo> {
    content: MaybeClosed<T>,
    marker: PhantomData<C>,
}

enum MaybeClosed<T> {
    Open(T),
    Closed,
}

impl<T, C: PyTypeInfo> Closable<T, C> {
    fn new(content: T) -> Self {
        Self {
            content: MaybeClosed::Open(content),
            marker: PhantomData,
        }
    }

    fn get(&self) -> PyResult<&T> {
        match &self.content {
            MaybeClosed::Open(content) => Ok(content),
            MaybeClosed::Closed => Err(PyValueError::new_err(format!(
                "The `{}` is closed",
                C::NAME,
            ))),
        }
    }

    fn close(&mut self) {
        if matches!(self.content, MaybeClosed::Open(_)) {
            debug!("Closing a {}", C::NAME);
        }
        self.content = MaybeClosed::Closed;
    }
}

impl<T, C: PyTypeInfo> Drop for Closable<T, C> {
    fn drop(&mut self) {
        self.close();
    }
}

#[pyfunction]
fn _validate_pronunciation(pronunciation: &str, py: Python<'_>) -> PyResult<()> {
    voicevox_core::__internal::validate_pronunciation(pronunciation).into_py_result(py)
}

#[pyfunction]
fn _to_zenkaku(text: &str) -> PyResult<String> {
    Ok(voicevox_core::__internal::to_zenkaku(text))
}

mod blocking {
    use std::{ffi::OsString, path::PathBuf, sync::Arc};

    use camino::Utf8PathBuf;
    use pyo3::{
        pyclass, pymethods,
        types::{IntoPyDict as _, PyBytes, PyDict, PyList},
        Py, PyAny, PyObject, PyRef, PyResult, Python,
    };
    use uuid::Uuid;
    use voicevox_core::{
        AccelerationMode, AudioQuery, InitializeOptions, StyleId, SynthesisOptions, TtsOptions,
        UserDictWord,
    };

    use crate::{convert::VoicevoxCoreResultExt as _, Closable};

    #[pyclass]
    #[derive(Clone)]
    pub(crate) struct VoiceModel {
        model: Arc<voicevox_core::blocking::VoiceModel>,
    }

    #[pymethods]
    impl VoiceModel {
        #[staticmethod]
        fn from_path(py: Python<'_>, path: PathBuf) -> PyResult<Self> {
            let model = voicevox_core::blocking::VoiceModel::from_path(path)
                .into_py_result(py)?
                .into();
            Ok(Self { model })
        }

        #[getter]
        fn id(&self, py: Python<'_>) -> PyResult<PyObject> {
            let id = self.model.id().raw_voice_model_id();
            crate::convert::to_py_uuid(py, id)
        }

        #[getter]
        fn metas<'py>(&self, py: Python<'py>) -> Vec<&'py PyAny> {
            crate::convert::to_pydantic_voice_model_meta(self.model.metas(), py).unwrap()
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
                        .exec()
                        .into_py_result(py)?;
                    Py::new(py, Self(inner))
                })
                .cloned()
        }

        fn supported_devices<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
            let class = py
                .import("voicevox_core")?
                .getattr("SupportedDevices")?
                .downcast()?;
            let s = self.0.supported_devices().into_py_result(py)?;
            crate::convert::to_pydantic_dataclass(s, class)
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
    pub(crate) struct Synthesizer {
        synthesizer: Closable<
            voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>,
            Self,
        >,
    }

    #[pymethods]
    impl Synthesizer {
        #[new]
        #[pyo3(signature =(
            onnxruntime,
            open_jtalk,
            acceleration_mode = InitializeOptions::default().acceleration_mode,
            cpu_num_threads = InitializeOptions::default().cpu_num_threads,
        ))]
        fn new(
            onnxruntime: Onnxruntime,
            open_jtalk: OpenJtalk,
            #[pyo3(from_py_with = "crate::convert::from_acceleration_mode")]
            acceleration_mode: AccelerationMode,
            cpu_num_threads: u16,
            py: Python<'_>,
        ) -> PyResult<Self> {
            let inner = voicevox_core::blocking::Synthesizer::new(
                onnxruntime.0,
                open_jtalk.open_jtalk.clone(),
                &InitializeOptions {
                    acceleration_mode,
                    cpu_num_threads,
                },
            )
            .into_py_result(py)?;
            Ok(Self {
                synthesizer: Closable::new(inner),
            })
        }

        fn __repr__(&self) -> &'static str {
            "Synthesizer { .. }"
        }

        fn __enter__(slf: PyRef<'_, Self>) -> PyResult<PyRef<'_, Self>> {
            slf.synthesizer.get()?;
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
            let synthesizer = self.synthesizer.get()?;
            Ok(synthesizer.is_gpu_mode())
        }

        #[getter]
        fn metas<'py>(&self, py: Python<'py>) -> PyResult<Vec<&'py PyAny>> {
            let synthesizer = self.synthesizer.get()?;
            crate::convert::to_pydantic_voice_model_meta(&synthesizer.metas(), py)
        }

        fn load_voice_model(&mut self, model: &PyAny, py: Python<'_>) -> PyResult<()> {
            let model: VoiceModel = model.extract()?;
            self.synthesizer
                .get()?
                .load_voice_model(&model.model)
                .into_py_result(py)
        }

        fn unload_voice_model(
            &mut self,
            #[pyo3(from_py_with = "crate::convert::to_rust_uuid")] voice_model_id: Uuid,
            py: Python<'_>,
        ) -> PyResult<()> {
            self.synthesizer
                .get()?
                .unload_voice_model(voice_model_id.into())
                .into_py_result(py)
        }

        fn is_loaded_voice_model(
            &self,
            #[pyo3(from_py_with = "crate::convert::to_rust_uuid")] voice_model_id: Uuid,
        ) -> PyResult<bool> {
            Ok(self
                .synthesizer
                .get()?
                .is_loaded_voice_model(voice_model_id.into()))
        }

        fn audio_query_from_kana<'py>(
            &self,
            kana: &str,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.get()?;

            let audio_query = synthesizer
                .audio_query_from_kana(kana, StyleId::new(style_id))
                .into_py_result(py)?;

            let class = py.import("voicevox_core")?.getattr("AudioQuery")?;
            crate::convert::to_pydantic_dataclass(audio_query, class)
        }

        fn audio_query<'py>(
            &self,
            text: &str,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizesr = self.synthesizer.get()?;

            let audio_query = synthesizesr
                .audio_query(text, StyleId::new(style_id))
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
            let synthesizer = self.synthesizer.get()?;

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
            let synthesizer = self.synthesizer.get()?;

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
            let synthesizer = self.synthesizer.get()?;
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
            let synthesizer = self.synthesizer.get()?;
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
            let synthesizer = self.synthesizer.get()?;
            crate::convert::blocking_modify_accent_phrases(
                accent_phrases,
                StyleId::new(style_id),
                py,
                |a, s| synthesizer.replace_mora_pitch(&a, s),
            )
        }

        #[pyo3(signature=(
            audio_query,
            style_id,
            enable_interrogative_upspeak = TtsOptions::default().enable_interrogative_upspeak
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
                .get()?
                .synthesis(
                    &audio_query,
                    StyleId::new(style_id),
                    &SynthesisOptions {
                        enable_interrogative_upspeak,
                    },
                )
                .into_py_result(py)?;
            Ok(PyBytes::new(py, wav))
        }

        #[pyo3(signature=(
            kana,
            style_id,
            enable_interrogative_upspeak = TtsOptions::default().enable_interrogative_upspeak
        ))]
        fn tts_from_kana<'py>(
            &self,
            kana: &str,
            style_id: u32,
            enable_interrogative_upspeak: bool,
            py: Python<'py>,
        ) -> PyResult<&'py PyBytes> {
            let style_id = StyleId::new(style_id);
            let options = &TtsOptions {
                enable_interrogative_upspeak,
            };
            let wav = &self
                .synthesizer
                .get()?
                .tts_from_kana(kana, style_id, options)
                .into_py_result(py)?;
            Ok(PyBytes::new(py, wav))
        }

        #[pyo3(signature=(
            text,
            style_id,
            enable_interrogative_upspeak = TtsOptions::default().enable_interrogative_upspeak
        ))]
        fn tts<'py>(
            &self,
            text: &str,
            style_id: u32,
            enable_interrogative_upspeak: bool,
            py: Python<'py>,
        ) -> PyResult<&'py PyBytes> {
            let style_id = StyleId::new(style_id);
            let options = &TtsOptions {
                enable_interrogative_upspeak,
            };
            let wav = &self
                .synthesizer
                .get()?
                .tts(text, style_id, options)
                .into_py_result(py)?;
            Ok(PyBytes::new(py, wav))
        }

        fn close(&mut self) {
            self.synthesizer.close()
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

        fn load(&self, path: &str, py: Python<'_>) -> PyResult<()> {
            self.dict.load(path).into_py_result(py)
        }

        fn save(&self, path: &str, py: Python<'_>) -> PyResult<()> {
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

        #[getter]
        fn words<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
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
        pyclass, pymethods,
        types::{IntoPyDict as _, PyBytes, PyDict, PyList},
        Py, PyAny, PyObject, PyRef, PyResult, Python, ToPyObject as _,
    };
    use uuid::Uuid;
    use voicevox_core::{
        AccelerationMode, AudioQuery, InitializeOptions, StyleId, SynthesisOptions, TtsOptions,
        UserDictWord,
    };

    use crate::{convert::VoicevoxCoreResultExt as _, Closable};

    #[pyclass]
    #[derive(Clone)]
    pub(crate) struct VoiceModel {
        model: Arc<voicevox_core::nonblocking::VoiceModel>,
    }

    #[pymethods]
    impl VoiceModel {
        #[staticmethod]
        fn from_path(py: Python<'_>, path: PathBuf) -> PyResult<&PyAny> {
            pyo3_asyncio::tokio::future_into_py(py, async move {
                let model = voicevox_core::nonblocking::VoiceModel::from_path(path).await;
                let model = Python::with_gil(|py| model.into_py_result(py))?.into();
                Ok(Self { model })
            })
        }

        #[getter]
        fn id(&self, py: Python<'_>) -> PyResult<PyObject> {
            let id = self.model.id().raw_voice_model_id();
            crate::convert::to_py_uuid(py, id)
        }

        #[getter]
        fn metas<'py>(&self, py: Python<'py>) -> Vec<&'py PyAny> {
            crate::convert::to_pydantic_voice_model_meta(self.model.metas(), py).unwrap()
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
                    .exec()
                    .await;

                ONNXRUNTIME.get_or_try_init(|| {
                    Python::with_gil(|py| Py::new(py, Self(inner.into_py_result(py)?)))
                })
            })
        }

        fn supported_devices<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
            let class = py
                .import("voicevox_core")?
                .getattr("SupportedDevices")?
                .downcast()?;
            let s = self.0.supported_devices().into_py_result(py)?;
            crate::convert::to_pydantic_dataclass(s, class)
        }
    }

    #[pyclass]
    #[derive(Clone)]
    pub(crate) struct OpenJtalk {
        open_jtalk: voicevox_core::nonblocking::OpenJtalk,
    }

    #[pymethods]
    impl OpenJtalk {
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
        synthesizer: Closable<
            voicevox_core::nonblocking::Synthesizer<voicevox_core::nonblocking::OpenJtalk>,
            Self,
        >,
    }

    #[pymethods]
    impl Synthesizer {
        #[new]
        #[pyo3(signature =(
            onnxruntime,
            open_jtalk,
            acceleration_mode = InitializeOptions::default().acceleration_mode,
            cpu_num_threads = InitializeOptions::default().cpu_num_threads,
        ))]
        fn new(
            onnxruntime: Onnxruntime,
            open_jtalk: OpenJtalk,
            #[pyo3(from_py_with = "crate::convert::from_acceleration_mode")]
            acceleration_mode: AccelerationMode,
            cpu_num_threads: u16,
        ) -> PyResult<Self> {
            let synthesizer = voicevox_core::nonblocking::Synthesizer::new(
                onnxruntime.0,
                open_jtalk.open_jtalk.clone(),
                &InitializeOptions {
                    acceleration_mode,
                    cpu_num_threads,
                },
            );
            let synthesizer = Python::with_gil(|py| synthesizer.into_py_result(py))?;
            let synthesizer = Closable::new(synthesizer);
            Ok(Self { synthesizer })
        }

        fn __repr__(&self) -> &'static str {
            "Synthesizer { .. }"
        }

        fn __enter__(slf: PyRef<'_, Self>) -> PyResult<PyRef<'_, Self>> {
            slf.synthesizer.get()?;
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
            let synthesizer = self.synthesizer.get()?;
            Ok(synthesizer.is_gpu_mode())
        }

        #[getter]
        fn metas<'py>(&self, py: Python<'py>) -> PyResult<Vec<&'py PyAny>> {
            let synthesizer = self.synthesizer.get()?;
            crate::convert::to_pydantic_voice_model_meta(&synthesizer.metas(), py)
        }

        fn load_voice_model<'py>(
            &mut self,
            model: &'py PyAny,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let model: VoiceModel = model.extract()?;
            let synthesizer = self.synthesizer.get()?.clone();
            pyo3_asyncio::tokio::future_into_py(py, async move {
                let result = synthesizer.load_voice_model(&model.model).await;
                Python::with_gil(|py| result.into_py_result(py))
            })
        }

        fn unload_voice_model(
            &mut self,
            #[pyo3(from_py_with = "crate::convert::to_rust_uuid")] voice_model_id: Uuid,
            py: Python<'_>,
        ) -> PyResult<()> {
            self.synthesizer
                .get()?
                .unload_voice_model(voice_model_id.into())
                .into_py_result(py)
        }

        fn is_loaded_voice_model(
            &self,
            #[pyo3(from_py_with = "crate::convert::to_rust_uuid")] voice_model_id: Uuid,
        ) -> PyResult<bool> {
            Ok(self
                .synthesizer
                .get()?
                .is_loaded_voice_model(voice_model_id.into()))
        }

        fn audio_query_from_kana<'py>(
            &self,
            kana: &str,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.get()?.clone();
            let kana = kana.to_owned();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let audio_query = synthesizer
                        .audio_query_from_kana(&kana, StyleId::new(style_id))
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

        fn audio_query<'py>(
            &self,
            text: &str,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.get()?.clone();
            let text = text.to_owned();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let audio_query = synthesizer.audio_query(&text, StyleId::new(style_id)).await;

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
            let synthesizer = self.synthesizer.get()?.clone();
            let kana = kana.to_owned();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let accent_phrases = synthesizer
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
            let synthesizer = self.synthesizer.get()?.clone();
            let text = text.to_owned();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let accent_phrases = synthesizer
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
            let synthesizer = self.synthesizer.get()?.clone();
            crate::convert::async_modify_accent_phrases(
                accent_phrases,
                StyleId::new(style_id),
                py,
                |a, s| async move { synthesizer.replace_mora_data(&a, s).await },
            )
        }

        fn replace_phoneme_length<'py>(
            &self,
            accent_phrases: &'py PyList,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.get()?.clone();
            crate::convert::async_modify_accent_phrases(
                accent_phrases,
                StyleId::new(style_id),
                py,
                |a, s| async move { synthesizer.replace_phoneme_length(&a, s).await },
            )
        }

        fn replace_mora_pitch<'py>(
            &self,
            accent_phrases: &'py PyList,
            style_id: u32,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.get()?.clone();
            crate::convert::async_modify_accent_phrases(
                accent_phrases,
                StyleId::new(style_id),
                py,
                |a, s| async move { synthesizer.replace_mora_pitch(&a, s).await },
            )
        }

        #[pyo3(signature=(audio_query,style_id,enable_interrogative_upspeak = TtsOptions::default().enable_interrogative_upspeak))]
        fn synthesis<'py>(
            &self,
            #[pyo3(from_py_with = "crate::convert::from_dataclass")] audio_query: AudioQuery,
            style_id: u32,
            enable_interrogative_upspeak: bool,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let synthesizer = self.synthesizer.get()?.clone();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let wav = synthesizer
                        .synthesis(
                            &audio_query,
                            StyleId::new(style_id),
                            &SynthesisOptions {
                                enable_interrogative_upspeak,
                            },
                        )
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
            enable_interrogative_upspeak = TtsOptions::default().enable_interrogative_upspeak
        ))]
        fn tts_from_kana<'py>(
            &self,
            kana: &str,
            style_id: u32,
            enable_interrogative_upspeak: bool,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let style_id = StyleId::new(style_id);
            let options = TtsOptions {
                enable_interrogative_upspeak,
            };
            let synthesizer = self.synthesizer.get()?.clone();
            let kana = kana.to_owned();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let wav = synthesizer.tts_from_kana(&kana, style_id, &options).await;

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
            enable_interrogative_upspeak = TtsOptions::default().enable_interrogative_upspeak
        ))]
        fn tts<'py>(
            &self,
            text: &str,
            style_id: u32,
            enable_interrogative_upspeak: bool,
            py: Python<'py>,
        ) -> PyResult<&'py PyAny> {
            let style_id = StyleId::new(style_id);
            let options = TtsOptions {
                enable_interrogative_upspeak,
            };
            let synthesizer = self.synthesizer.get()?.clone();
            let text = text.to_owned();
            pyo3_asyncio::tokio::future_into_py_with_locals(
                py,
                pyo3_asyncio::tokio::get_current_locals(py)?,
                async move {
                    let wav = synthesizer.tts(&text, style_id, &options).await;

                    Python::with_gil(|py| {
                        let wav = wav.into_py_result(py)?;
                        Ok(PyBytes::new(py, &wav).to_object(py))
                    })
                },
            )
        }

        fn close(&mut self) {
            self.synthesizer.close()
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

        fn load<'py>(&self, path: &str, py: Python<'py>) -> PyResult<&'py PyAny> {
            let this = self.dict.clone();
            let path = path.to_owned();

            pyo3_asyncio::tokio::future_into_py(py, async move {
                let result = this.load(&path).await;
                Python::with_gil(|py| result.into_py_result(py))
            })
        }

        fn save<'py>(&self, path: &str, py: Python<'py>) -> PyResult<&'py PyAny> {
            let this = self.dict.clone();
            let path = path.to_owned();

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

        #[getter]
        fn words<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
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
