use std::{marker::PhantomData, sync::Arc};

mod convert;
use convert::*;
use log::debug;
use pyo3::{
    create_exception,
    exceptions::{PyException, PyKeyError, PyValueError},
    pyclass, pyfunction, pymethods, pymodule,
    types::{IntoPyDict as _, PyBytes, PyDict, PyList, PyModule},
    wrap_pyfunction, PyAny, PyObject, PyRef, PyResult, PyTypeInfo, Python, ToPyObject,
};
use uuid::Uuid;
use voicevox_core::{
    AccelerationMode, AudioQueryModel, InitializeOptions, StyleId, SynthesisOptions, TtsOptions,
    UserDictWord, VoiceModelId,
};

#[pymodule]
#[pyo3(name = "_rust")]
fn rust(_: Python<'_>, module: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_wrapped(wrap_pyfunction!(supported_devices))?;
    module.add_wrapped(wrap_pyfunction!(_validate_pronunciation))?;
    module.add_wrapped(wrap_pyfunction!(_to_zenkaku))?;

    module.add_class::<Synthesizer>()?;
    module.add_class::<OpenJtalk>()?;
    module.add_class::<VoiceModel>()?;
    module.add_class::<UserDict>()?;

    add_exceptions(module)
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
    OpenZipFileError: PyException;
    ReadZipEntryError: PyException;
    ModelAlreadyLoadedError: PyException;
    StyleAlreadyLoadedError: PyException;
    InvalidModelDataError: PyException;
    GetSupportedDevicesError: PyException;
    StyleNotFoundError: PyKeyError;
    ModelNotFoundError: PyKeyError;
    InferenceFailedError: PyException;
    ExtractFullContextLabelError: PyException;
    ParseKanaError: PyValueError;
    LoadUserDictError: PyException;
    SaveUserDictError: PyException;
    WordNotFoundError: PyKeyError;
    UseUserDictError: PyException;
    InvalidWordError: PyValueError;
}

#[pyclass]
#[derive(Clone)]
struct VoiceModel {
    model: voicevox_core::tokio::VoiceModel,
}

#[pyfunction]
fn supported_devices(py: Python<'_>) -> PyResult<&PyAny> {
    let class = py
        .import("voicevox_core")?
        .getattr("SupportedDevices")?
        .downcast()?;
    let s = voicevox_core::SupportedDevices::create().into_py_result(py)?;
    to_pydantic_dataclass(s, class)
}

#[pymethods]
impl VoiceModel {
    #[staticmethod]
    fn from_path(
        py: Python<'_>,
        #[pyo3(from_py_with = "from_utf8_path")] path: String,
    ) -> PyResult<&PyAny> {
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let model = voicevox_core::tokio::VoiceModel::from_path(path).await;
            let model = Python::with_gil(|py| model.into_py_result(py))?;
            Ok(Self { model })
        })
    }

    #[getter]
    fn id(&self) -> &str {
        self.model.id().raw_voice_model_id()
    }

    #[getter]
    fn metas<'py>(&self, py: Python<'py>) -> Vec<&'py PyAny> {
        to_pydantic_voice_model_meta(self.model.metas(), py).unwrap()
    }
}

#[pyclass]
#[derive(Clone)]
struct OpenJtalk {
    open_jtalk: voicevox_core::tokio::OpenJtalk,
}

#[pymethods]
impl OpenJtalk {
    #[allow(clippy::new_ret_no_self)]
    #[staticmethod]
    fn new(
        #[pyo3(from_py_with = "from_utf8_path")] open_jtalk_dict_dir: String,
        py: Python<'_>,
    ) -> PyResult<&PyAny> {
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let open_jtalk = voicevox_core::tokio::OpenJtalk::new(open_jtalk_dict_dir).await;
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
struct Synthesizer {
    synthesizer: Closable<voicevox_core::tokio::Synthesizer<voicevox_core::tokio::OpenJtalk>, Self>,
}

#[pymethods]
impl Synthesizer {
    #[new]
    #[pyo3(signature =(
        open_jtalk,
        acceleration_mode = InitializeOptions::default().acceleration_mode,
        cpu_num_threads = InitializeOptions::default().cpu_num_threads,
    ))]
    fn new(
        open_jtalk: OpenJtalk,
        #[pyo3(from_py_with = "from_acceleration_mode")] acceleration_mode: AccelerationMode,
        cpu_num_threads: u16,
    ) -> PyResult<Self> {
        let synthesizer = voicevox_core::tokio::Synthesizer::new(
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
        #[allow(unused_variables)] exc_type: &PyAny,
        #[allow(unused_variables)] exc_value: &PyAny,
        #[allow(unused_variables)] traceback: &PyAny,
    ) {
        self.close();
    }

    #[getter]
    fn is_gpu_mode(&self) -> PyResult<bool> {
        let synthesizer = self.synthesizer.get()?;
        Ok(synthesizer.is_gpu_mode())
    }

    #[getter]
    fn metas<'py>(&self, py: Python<'py>) -> PyResult<Vec<&'py PyAny>> {
        let synthesizer = self.synthesizer.get()?;
        to_pydantic_voice_model_meta(&synthesizer.metas(), py)
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

    fn unload_voice_model(&mut self, voice_model_id: &str, py: Python<'_>) -> PyResult<()> {
        self.synthesizer
            .get()?
            .unload_voice_model(&VoiceModelId::new(voice_model_id.to_string()))
            .into_py_result(py)
    }

    fn is_loaded_voice_model(&self, voice_model_id: &str) -> PyResult<bool> {
        Ok(self
            .synthesizer
            .get()?
            .is_loaded_voice_model(&VoiceModelId::new(voice_model_id.to_string())))
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
                    let ret = to_pydantic_dataclass(audio_query.into_py_result(py)?, class)?;
                    Ok(ret.to_object(py))
                })
            },
        )
    }

    fn audio_query<'py>(&self, text: &str, style_id: u32, py: Python<'py>) -> PyResult<&'py PyAny> {
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
                    let ret = to_pydantic_dataclass(audio_query, class)?;
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
                        .map(|ap| to_pydantic_dataclass(ap, class))
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
                        .map(|ap| to_pydantic_dataclass(ap, class))
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
        modify_accent_phrases(
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
        modify_accent_phrases(
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
        modify_accent_phrases(
            accent_phrases,
            StyleId::new(style_id),
            py,
            |a, s| async move { synthesizer.replace_mora_pitch(&a, s).await },
        )
    }

    #[pyo3(signature=(audio_query,style_id,enable_interrogative_upspeak = TtsOptions::default().enable_interrogative_upspeak))]
    fn synthesis<'py>(
        &self,
        #[pyo3(from_py_with = "from_dataclass")] audio_query: AudioQueryModel,
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

#[pyclass]
#[derive(Default, Debug, Clone)]
struct UserDict {
    dict: Arc<voicevox_core::tokio::UserDict>,
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
        #[pyo3(from_py_with = "to_rust_user_dict_word")] word: UserDictWord,
        py: Python<'_>,
    ) -> PyResult<PyObject> {
        let uuid = self.dict.add_word(word).into_py_result(py)?;

        to_py_uuid(py, uuid)
    }

    fn update_word(
        &mut self,
        #[pyo3(from_py_with = "to_rust_uuid")] word_uuid: Uuid,
        #[pyo3(from_py_with = "to_rust_user_dict_word")] word: UserDictWord,
        py: Python<'_>,
    ) -> PyResult<()> {
        self.dict.update_word(word_uuid, word).into_py_result(py)?;
        Ok(())
    }

    fn remove_word(
        &mut self,
        #[pyo3(from_py_with = "to_rust_uuid")] word_uuid: Uuid,
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
                    let uuid = to_py_uuid(py, uuid)?;
                    let word = to_py_user_dict_word(py, word)?;
                    Ok((uuid, word))
                })
                .collect::<PyResult<Vec<_>>>()
        })?;
        Ok(words.into_py_dict(py))
    }
}
