use std::{fmt::Display, path::PathBuf};

use easy_ext::ext;
use log::debug;
use numpy::{Ix1, PyArray};
use pyo3::{
    create_exception,
    exceptions::PyException,
    pyclass, pymethods, pymodule,
    types::{PyBytes, PyModule},
    FromPyObject as _, PyAny, PyResult, Python,
};
use serde::{de::DeserializeOwned, Serialize};
use voicevox_core::{
    AccelerationMode, AccentPhraseModel, AccentPhrasesOptions, AudioQueryModel, AudioQueryOptions,
    InitializeOptions, SynthesisOptions, TtsOptions,
};

#[pymodule]
#[pyo3(name = "_rust")]
fn rust(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    pyo3_log::init();

    module.add("METAS", {
        let class = py.import("voicevox_core")?.getattr("Meta")?.cast_as()?;
        let meta_from_json = |x: &serde_json::Value| to_pydantic_dataclass(x, class);
        serde_json::from_str::<Vec<_>>(voicevox_core::METAS)
            .into_py_result()?
            .into_iter()
            .map(|meta| meta_from_json(&meta))
            .collect::<Result<Vec<_>, _>>()?
    })?;

    module.add("SUPPORTED_DEVICES", {
        let class = py
            .import("voicevox_core")?
            .getattr("SupportedDevices")?
            .cast_as()?;
        let supported_devices_from_json = |x: &serde_json::Value| to_pydantic_dataclass(x, class);
        supported_devices_from_json(&voicevox_core::SUPPORTED_DEVICES.to_json())?
    })?;

    module.add("__version__", voicevox_core::VoicevoxCore::get_version())?;

    module.add_class::<VoicevoxCore>()
}

create_exception!(
    voicevox_core,
    VoicevoxError,
    PyException,
    "voicevox_core Error."
);

#[pyclass]
struct VoicevoxCore {
    inner: voicevox_core::VoicevoxCore,
}

#[pymethods]
impl VoicevoxCore {
    #[new]
    #[args(
        acceleration_mode = "InitializeOptions::default().acceleration_mode",
        cpu_num_threads = "InitializeOptions::default().cpu_num_threads",
        load_all_models = "InitializeOptions::default().load_all_models",
        open_jtalk_dict_dir = "None"
    )]
    fn new(
        #[pyo3(from_py_with = "from_acceleration_mode")] acceleration_mode: AccelerationMode,
        cpu_num_threads: u16,
        load_all_models: bool,
        #[pyo3(from_py_with = "from_optional_utf8_path")] open_jtalk_dict_dir: Option<String>,
    ) -> PyResult<Self> {
        let inner = voicevox_core::VoicevoxCore::new_with_initialize(InitializeOptions {
            acceleration_mode,
            cpu_num_threads,
            load_all_models,
            open_jtalk_dict_dir: open_jtalk_dict_dir.map(Into::into),
        })
        .into_py_result()?;
        Ok(Self { inner })
    }

    fn __repr__(&self) -> &'static str {
        "VoicevoxCore { .. }"
    }

    #[getter]
    fn is_gpu_mode(&self) -> bool {
        self.inner.is_gpu_mode()
    }

    fn load_model(&mut self, speaker_id: u32) -> PyResult<()> {
        self.inner.load_model(speaker_id).into_py_result()
    }

    fn is_model_loaded(&self, speaker_id: u32) -> bool {
        self.inner.is_model_loaded(speaker_id)
    }

    fn predict_duration<'py>(
        &mut self,
        phoneme_vector: &'py PyArray<i64, Ix1>,
        speaker_id: u32,
        py: Python<'py>,
    ) -> PyResult<&'py PyArray<f32, Ix1>> {
        let duration = self
            .inner
            .predict_duration(&phoneme_vector.to_vec()?, speaker_id)
            .into_py_result()?;
        Ok(PyArray::from_vec(py, duration))
    }

    #[allow(clippy::too_many_arguments)]
    fn predict_intonation<'py>(
        &mut self,
        length: usize,
        vowel_phoneme_vector: &'py PyArray<i64, Ix1>,
        consonant_phoneme_vector: &'py PyArray<i64, Ix1>,
        start_accent_vector: &'py PyArray<i64, Ix1>,
        end_accent_vector: &'py PyArray<i64, Ix1>,
        start_accent_phrase_vector: &'py PyArray<i64, Ix1>,
        end_accent_phrase_vector: &'py PyArray<i64, Ix1>,
        speaker_id: u32,
        py: Python<'py>,
    ) -> PyResult<&'py PyArray<f32, Ix1>> {
        let intonation = self
            .inner
            .predict_intonation(
                length,
                &vowel_phoneme_vector.to_vec()?,
                &consonant_phoneme_vector.to_vec()?,
                &start_accent_vector.to_vec()?,
                &end_accent_vector.to_vec()?,
                &start_accent_phrase_vector.to_vec()?,
                &end_accent_phrase_vector.to_vec()?,
                speaker_id,
            )
            .into_py_result()?;
        Ok(PyArray::from_vec(py, intonation))
    }

    fn decode<'py>(
        &mut self,
        length: usize,
        phoneme_size: usize,
        f0: &'py PyArray<f32, Ix1>,
        phoneme: &'py PyArray<f32, Ix1>,
        speaker_id: u32,
        py: Python<'py>,
    ) -> PyResult<&'py PyArray<f32, Ix1>> {
        let decoded = self
            .inner
            .decode(
                length,
                phoneme_size,
                &f0.to_vec()?,
                &phoneme.to_vec()?,
                speaker_id,
            )
            .into_py_result()?;
        Ok(PyArray::from_vec(py, decoded))
    }

    #[args(kana = "AudioQueryOptions::default().kana")]
    fn audio_query<'py>(
        &mut self,
        text: &str,
        speaker_id: u32,
        kana: bool,
        py: Python<'py>,
    ) -> PyResult<&'py PyAny> {
        let audio_query = &self
            .inner
            .audio_query(text, speaker_id, AudioQueryOptions { kana })
            .into_py_result()?;
        to_pydantic_dataclass(
            audio_query,
            py.import("voicevox_core")?.getattr("AudioQuery")?,
        )
    }

    #[args(kana = "AccentPhrasesOptions::default().kana")]
    fn accent_phrases<'py>(
        &mut self,
        text: &str,
        speaker_id: u32,
        kana: bool,
        py: Python<'py>,
    ) -> PyResult<Vec<&'py PyAny>> {
        let accent_phrases = &self
            .inner
            .accent_phrases(text, speaker_id, AccentPhrasesOptions { kana })
            .into_py_result()?;
        accent_phrases
            .iter()
            .map(|accent_phrase| {
                to_pydantic_dataclass(
                    accent_phrase,
                    py.import("voicevox_core")?.getattr("AccentPhrase")?,
                )
            })
            .collect()
    }

    fn mora_length<'py>(
        &mut self,
        accent_phrases: Vec<&'py PyAny>,
        speaker_id: u32,
        py: Python<'py>,
    ) -> PyResult<Vec<&'py PyAny>> {
        modify_accent_phrases(accent_phrases, speaker_id, py, |a, s| {
            self.inner.mora_length(a, s)
        })
    }

    fn mora_pitch<'py>(
        &mut self,
        accent_phrases: Vec<&'py PyAny>,
        speaker_id: u32,
        py: Python<'py>,
    ) -> PyResult<Vec<&'py PyAny>> {
        modify_accent_phrases(accent_phrases, speaker_id, py, |a, s| {
            self.inner.mora_pitch(a, s)
        })
    }

    fn mora_data<'py>(
        &mut self,
        accent_phrases: Vec<&'py PyAny>,
        speaker_id: u32,
        py: Python<'py>,
    ) -> PyResult<Vec<&'py PyAny>> {
        modify_accent_phrases(accent_phrases, speaker_id, py, |a, s| {
            self.inner.mora_data(a, s)
        })
    }

    #[args(enable_interrogative_upspeak = "TtsOptions::default().enable_interrogative_upspeak")]
    fn synthesis<'py>(
        &mut self,
        #[pyo3(from_py_with = "from_dataclass")] audio_query: AudioQueryModel,
        speaker_id: u32,
        enable_interrogative_upspeak: bool,
        py: Python<'py>,
    ) -> PyResult<&'py PyBytes> {
        let wav = &self
            .inner
            .synthesis(
                &audio_query,
                speaker_id,
                SynthesisOptions {
                    enable_interrogative_upspeak,
                },
            )
            .into_py_result()?;
        Ok(PyBytes::new(py, wav))
    }

    #[args(
        kana = "TtsOptions::default().kana",
        enable_interrogative_upspeak = "TtsOptions::default().enable_interrogative_upspeak"
    )]
    fn tts<'py>(
        &mut self,
        text: &str,
        speaker_id: u32,
        kana: bool,
        enable_interrogative_upspeak: bool,
        py: Python<'py>,
    ) -> PyResult<&'py PyBytes> {
        let wav = &self
            .inner
            .tts(
                text,
                speaker_id,
                TtsOptions {
                    kana,
                    enable_interrogative_upspeak,
                },
            )
            .into_py_result()?;
        Ok(PyBytes::new(py, wav))
    }
}

fn from_acceleration_mode(ob: &PyAny) -> PyResult<AccelerationMode> {
    let py = ob.py();

    let class = py.import("voicevox_core")?.getattr("AccelerationMode")?;
    let mode = class.get_item(ob)?;

    if mode.eq(class.getattr("AUTO")?)? {
        Ok(AccelerationMode::Auto)
    } else if mode.eq(class.getattr("CPU")?)? {
        Ok(AccelerationMode::Cpu)
    } else if mode.eq(class.getattr("GPU")?)? {
        Ok(AccelerationMode::Gpu)
    } else {
        unreachable!("{} should be one of {{AUTO, CPU, GPU}}", mode.repr()?);
    }
}

fn from_optional_utf8_path(ob: &PyAny) -> PyResult<Option<String>> {
    if ob.is_none() {
        return Ok(None);
    }

    PathBuf::extract(ob)?
        .into_os_string()
        .into_string()
        .map(Some)
        .map_err(|s| VoicevoxError::new_err(format!("{s:?} cannot be encoded to UTF-8")))
}

fn from_dataclass<T: DeserializeOwned>(ob: &PyAny) -> PyResult<T> {
    let py = ob.py();

    let ob = py.import("dataclasses")?.call_method1("asdict", (ob,))?;
    let json = &py
        .import("json")?
        .call_method1("dumps", (ob,))?
        .extract::<String>()?;
    serde_json::from_str(json).into_py_result()
}

fn to_pydantic_dataclass(x: impl Serialize, class: &PyAny) -> PyResult<&PyAny> {
    let py = class.py();

    let x = serde_json::to_string(&x).into_py_result()?;
    let x = py.import("json")?.call_method1("loads", (x,))?.cast_as()?;
    class.call((), Some(x))
}

fn modify_accent_phrases<'py, F>(
    accent_phrases: Vec<&'py PyAny>,
    speaker_id: u32,
    py: Python<'py>,
    mut method: F,
) -> PyResult<Vec<&'py PyAny>>
where
    F: FnMut(u32, &[AccentPhraseModel]) -> Result<Vec<AccentPhraseModel>, voicevox_core::Error>,
{
    let rust_accent_phrases = accent_phrases
        .iter()
        .map(|a| from_dataclass::<AccentPhraseModel>(a))
        .collect::<PyResult<Vec<_>>>()?;
    let replaced_accent_phrases = method(speaker_id, &rust_accent_phrases).into_py_result()?;
    replaced_accent_phrases
        .iter()
        .map(move |accent_phrase| {
            to_pydantic_dataclass(
                accent_phrase,
                py.import("voicevox_core")?.getattr("AccentPhrase")?,
            )
        })
        .collect()
}

impl Drop for VoicevoxCore {
    fn drop(&mut self) {
        debug!("Destructing a VoicevoxCore");
        self.inner.finalize();
    }
}

#[ext]
impl<T, E: Display> Result<T, E> {
    fn into_py_result(self) -> PyResult<T> {
        self.map_err(|e| VoicevoxError::new_err(e.to_string()))
    }
}
