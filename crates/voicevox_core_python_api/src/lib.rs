use std::sync::Arc;

mod convert;
use convert::*;
use log::debug;
use once_cell::sync::Lazy;
use pyo3::{
    create_exception,
    exceptions::PyException,
    pyclass, pyfunction, pymethods, pymodule,
    types::{IntoPyDict as _, PyBytes, PyDict, PyList, PyModule},
    wrap_pyfunction, PyAny, PyObject, PyResult, Python, ToPyObject,
};
use tokio::{runtime::Runtime, sync::Mutex};
use uuid::Uuid;
use voicevox_core::{
    AccelerationMode, AccentPhrasesOptions, AudioQueryModel, AudioQueryOptions, InitializeOptions,
    StyleId, SynthesisOptions, TtsOptions, UserDictWord, VoiceModelId,
};

static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

#[pymodule]
#[pyo3(name = "_rust")]
fn rust(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_wrapped(wrap_pyfunction!(supported_devices))?;
    module.add_wrapped(wrap_pyfunction!(_validate_pronunciation))?;
    module.add_wrapped(wrap_pyfunction!(_to_zenkaku))?;

    module.add_class::<Synthesizer>()?;
    module.add_class::<OpenJtalk>()?;
    module.add_class::<VoiceModel>()?;
    module.add_class::<UserDict>()?;
    module.add("VoicevoxError", py.get_type::<VoicevoxError>())?;
    Ok(())
}

create_exception!(
    voicevox_core,
    VoicevoxError,
    PyException,
    "voicevox_core Error."
);

#[pyclass]
#[derive(Clone)]
struct VoiceModel {
    model: voicevox_core::VoiceModel,
}

#[pyfunction]
fn supported_devices(py: Python) -> PyResult<&PyAny> {
    let class = py
        .import("voicevox_core")?
        .getattr("SupportedDevices")?
        .downcast()?;
    let s = voicevox_core::SupportedDevices::create().into_py_result()?;
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
            let model = voicevox_core::VoiceModel::from_path(path)
                .await
                .into_py_result()?;
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
    open_jtalk: Arc<voicevox_core::OpenJtalk>,
}

#[pymethods]
impl OpenJtalk {
    #[new]
    fn new(#[pyo3(from_py_with = "from_utf8_path")] open_jtalk_dict_dir: String) -> PyResult<Self> {
        Ok(Self {
            open_jtalk: Arc::new(
                voicevox_core::OpenJtalk::new_with_initialize(open_jtalk_dict_dir)
                    .into_py_result()?,
            ),
        })
    }

    fn use_user_dict(&self, user_dict: UserDict) -> PyResult<()> {
        self.open_jtalk
            .use_user_dict(&user_dict.dict)
            .into_py_result()
    }
}

#[pyclass]
struct Synthesizer {
    synthesizer: Arc<Mutex<voicevox_core::Synthesizer>>,
}

#[pymethods]
impl Synthesizer {
    #[staticmethod]
    #[pyo3(signature =(
        open_jtalk,
        acceleration_mode = InitializeOptions::default().acceleration_mode,
        cpu_num_threads = InitializeOptions::default().cpu_num_threads,
        load_all_models = InitializeOptions::default().load_all_models,
    ))]
    fn new_with_initialize(
        py: Python,
        open_jtalk: OpenJtalk,
        #[pyo3(from_py_with = "from_acceleration_mode")] acceleration_mode: AccelerationMode,
        cpu_num_threads: u16,
        load_all_models: bool,
    ) -> PyResult<&PyAny> {
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let synthesizer = voicevox_core::Synthesizer::new_with_initialize(
                open_jtalk.open_jtalk.clone(),
                &InitializeOptions {
                    acceleration_mode,
                    cpu_num_threads,
                    load_all_models,
                },
            )
            .await
            .into_py_result()?;
            Ok(Self {
                synthesizer: Arc::new(Mutex::new(synthesizer)),
            })
        })
    }

    fn __repr__(&self) -> &'static str {
        "Synthesizer { .. }"
    }

    #[getter]
    fn is_gpu_mode(&self) -> bool {
        RUNTIME.block_on(self.synthesizer.lock()).is_gpu_mode()
    }

    #[getter]
    fn metas<'py>(&self, py: Python<'py>) -> Vec<&'py PyAny> {
        to_pydantic_voice_model_meta(&RUNTIME.block_on(self.synthesizer.lock()).metas(), py)
            .unwrap()
    }

    fn load_voice_model<'py>(
        &mut self,
        model: &'py PyAny,
        py: Python<'py>,
    ) -> PyResult<&'py PyAny> {
        let model: VoiceModel = model.extract()?;
        let synthesizer = self.synthesizer.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            synthesizer
                .lock()
                .await
                .load_voice_model(&model.model)
                .await
                .into_py_result()
        })
    }

    fn unload_voice_model(&mut self, voice_model_id: &str) -> PyResult<()> {
        RUNTIME
            .block_on(self.synthesizer.lock())
            .unload_voice_model(&VoiceModelId::new(voice_model_id.to_string()))
            .into_py_result()
    }

    fn is_loaded_voice_model(&self, voice_model_id: &str) -> bool {
        RUNTIME
            .block_on(self.synthesizer.lock())
            .is_loaded_voice_model(&VoiceModelId::new(voice_model_id.to_string()))
    }

    #[pyo3(signature=(text,style_id,kana = AudioQueryOptions::default().kana))]
    fn audio_query<'py>(
        &self,
        text: &str,
        style_id: u32,
        kana: bool,
        py: Python<'py>,
    ) -> PyResult<&'py PyAny> {
        let synthesizer = self.synthesizer.clone();
        let text = text.to_owned();
        pyo3_asyncio::tokio::future_into_py_with_locals(
            py,
            pyo3_asyncio::tokio::get_current_locals(py)?,
            async move {
                let audio_query = synthesizer
                    .lock()
                    .await
                    .audio_query(&text, StyleId::new(style_id), &AudioQueryOptions { kana })
                    .await
                    .into_py_result()?;

                Python::with_gil(|py| {
                    let class = py.import("voicevox_core")?.getattr("AudioQuery")?;
                    let ret = to_pydantic_dataclass(audio_query, class)?;
                    Ok(ret.to_object(py))
                })
            },
        )
    }

    #[pyo3(signature=(text, style_id, kana = AccentPhrasesOptions::default().kana))]
    fn create_accent_phrases<'py>(
        &self,
        text: &str,
        style_id: u32,
        kana: bool,
        py: Python<'py>,
    ) -> PyResult<&'py PyAny> {
        let synthesizer = self.synthesizer.clone();
        let text = text.to_owned();
        pyo3_asyncio::tokio::future_into_py_with_locals(
            py,
            pyo3_asyncio::tokio::get_current_locals(py)?,
            async move {
                let accent_phrases = synthesizer
                    .lock()
                    .await
                    .create_accent_phrases(
                        &text,
                        StyleId::new(style_id),
                        &AccentPhrasesOptions { kana },
                    )
                    .await
                    .into_py_result()?;
                Python::with_gil(|py| {
                    let class = py.import("voicevox_core")?.getattr("AccentPhrase")?;
                    let accent_phrases = accent_phrases
                        .iter()
                        .map(|ap| to_pydantic_dataclass(ap, class))
                        .collect::<PyResult<Vec<_>>>();
                    let list = PyList::new(py, accent_phrases.into_iter());
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
        modify_accent_phrases(
            accent_phrases,
            StyleId::new(style_id),
            py,
            |a, s| async move { synthesizer.lock().await.replace_mora_data(&a, s).await },
        )
    }

    fn replace_phoneme_length<'py>(
        &self,
        accent_phrases: &'py PyList,
        style_id: u32,
        py: Python<'py>,
    ) -> PyResult<&'py PyAny> {
        let synthesizer = self.synthesizer.clone();
        modify_accent_phrases(
            accent_phrases,
            StyleId::new(style_id),
            py,
            |a, s| async move { synthesizer.lock().await.replace_phoneme_length(&a, s).await },
        )
    }

    fn replace_mora_pitch<'py>(
        &self,
        accent_phrases: &'py PyList,
        style_id: u32,
        py: Python<'py>,
    ) -> PyResult<&'py PyAny> {
        let synthesizer = self.synthesizer.clone();
        modify_accent_phrases(
            accent_phrases,
            StyleId::new(style_id),
            py,
            |a, s| async move { synthesizer.lock().await.replace_mora_pitch(&a, s).await },
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
        let synthesizer = self.synthesizer.clone();
        pyo3_asyncio::tokio::future_into_py_with_locals(
            py,
            pyo3_asyncio::tokio::get_current_locals(py)?,
            async move {
                let wav = synthesizer
                    .lock()
                    .await
                    .synthesis(
                        &audio_query,
                        StyleId::new(style_id),
                        &SynthesisOptions {
                            enable_interrogative_upspeak,
                        },
                    )
                    .await
                    .into_py_result()?;
                Python::with_gil(|py| Ok(PyBytes::new(py, &wav).to_object(py)))
            },
        )
    }

    #[pyo3(signature=(
        text,
        style_id,
        kana = TtsOptions::default().kana,
        enable_interrogative_upspeak = TtsOptions::default().enable_interrogative_upspeak
    ))]
    fn tts<'py>(
        &self,
        text: &str,
        style_id: u32,
        kana: bool,
        enable_interrogative_upspeak: bool,
        py: Python<'py>,
    ) -> PyResult<&'py PyAny> {
        let style_id = StyleId::new(style_id);
        let options = TtsOptions {
            kana,
            enable_interrogative_upspeak,
        };
        let synthesizer = self.synthesizer.clone();
        let text = text.to_owned();
        pyo3_asyncio::tokio::future_into_py_with_locals(
            py,
            pyo3_asyncio::tokio::get_current_locals(py)?,
            async move {
                let wav = synthesizer
                    .lock()
                    .await
                    .tts(&text, style_id, &options)
                    .await
                    .into_py_result()?;
                Python::with_gil(|py| Ok(PyBytes::new(py, &wav).to_object(py)))
            },
        )
    }
}

#[pyfunction]
fn _validate_pronunciation(pronunciation: &str) -> PyResult<()> {
    voicevox_core::validate_pronunciation(pronunciation).into_py_result()
}

#[pyfunction]
fn _to_zenkaku(text: &str) -> PyResult<String> {
    Ok(voicevox_core::to_zenkaku(text))
}

#[pyclass]
#[derive(Default, Debug, Clone)]
struct UserDict {
    dict: voicevox_core::UserDict,
}

#[pymethods]
impl UserDict {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    fn load(&mut self, path: &str) -> PyResult<()> {
        self.dict.load(path).into_py_result()
    }

    fn save(&self, path: &str) -> PyResult<()> {
        self.dict.save(path).into_py_result()
    }

    fn add_word(
        &mut self,
        #[pyo3(from_py_with = "to_rust_user_dict_word")] word: UserDictWord,
        py: Python,
    ) -> PyResult<PyObject> {
        let uuid = self.dict.add_word(word).into_py_result()?;

        to_py_uuid(py, uuid)
    }

    fn update_word(
        &mut self,
        #[pyo3(from_py_with = "to_rust_uuid")] word_uuid: Uuid,
        #[pyo3(from_py_with = "to_rust_user_dict_word")] word: UserDictWord,
    ) -> PyResult<()> {
        self.dict.update_word(word_uuid, word).into_py_result()?;
        Ok(())
    }

    fn remove_word(
        &mut self,
        #[pyo3(from_py_with = "to_rust_uuid")] word_uuid: Uuid,
    ) -> PyResult<()> {
        self.dict.remove_word(word_uuid).into_py_result()?;
        Ok(())
    }

    fn import_dict(&mut self, other: &UserDict) -> PyResult<()> {
        self.dict.import(&other.dict).into_py_result()?;
        Ok(())
    }

    #[getter]
    fn words<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let words = self
            .dict
            .words()
            .iter()
            .map(|(&uuid, word)| {
                let uuid = to_py_uuid(py, uuid)?;
                let word = to_py_user_dict_word(py, word)?;
                Ok((uuid, word))
            })
            .collect::<PyResult<Vec<_>>>()?;
        Ok(words.into_py_dict(py))
    }
}

impl Drop for Synthesizer {
    fn drop(&mut self) {
        debug!("Destructing a VoicevoxCore");
    }
}
