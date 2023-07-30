use crate::VoicevoxError;
use std::{fmt::Display, future::Future, path::PathBuf};

use easy_ext::ext;
use pyo3::{types::PyList, FromPyObject as _, PyAny, PyObject, PyResult, Python, ToPyObject};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use uuid::Uuid;
use voicevox_core::{
    AccelerationMode, AccentPhraseModel, StyleId, UserDictWordType, VoiceModelMeta,
};

pub fn from_acceleration_mode(ob: &PyAny) -> PyResult<AccelerationMode> {
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

pub fn from_utf8_path(ob: &PyAny) -> PyResult<String> {
    PathBuf::extract(ob)?
        .into_os_string()
        .into_string()
        .map_err(|s| VoicevoxError::new_err(format!("{s:?} cannot be encoded to UTF-8")))
}

pub fn from_dataclass<T: DeserializeOwned>(ob: &PyAny) -> PyResult<T> {
    let py = ob.py();

    let ob = py.import("dataclasses")?.call_method1("asdict", (ob,))?;
    let json = &py
        .import("json")?
        .call_method1("dumps", (ob,))?
        .extract::<String>()?;
    serde_json::from_str(json).into_py_result()
}

pub fn to_pydantic_voice_model_meta<'py>(
    metas: &VoiceModelMeta,
    py: Python<'py>,
) -> PyResult<Vec<&'py PyAny>> {
    let class = py
        .import("voicevox_core")?
        .getattr("SpeakerMeta")?
        .downcast()?;

    metas
        .iter()
        .map(|m| to_pydantic_dataclass(m, class))
        .collect::<PyResult<Vec<_>>>()
}

pub fn to_pydantic_dataclass(x: impl Serialize, class: &PyAny) -> PyResult<&PyAny> {
    let py = class.py();

    let x = serde_json::to_string(&x).into_py_result()?;
    let x = py.import("json")?.call_method1("loads", (x,))?.downcast()?;
    class.call((), Some(x))
}

pub fn modify_accent_phrases<'py, Fun, Fut>(
    accent_phrases: &'py PyList,
    speaker_id: StyleId,
    py: Python<'py>,
    method: Fun,
) -> PyResult<&'py PyAny>
where
    Fun: FnOnce(Vec<AccentPhraseModel>, StyleId) -> Fut + Send + 'static,
    Fut: Future<Output = voicevox_core::Result<Vec<AccentPhraseModel>>> + Send + 'static,
{
    let rust_accent_phrases = accent_phrases
        .iter()
        .map(from_dataclass)
        .collect::<PyResult<Vec<AccentPhraseModel>>>()?;
    pyo3_asyncio::tokio::future_into_py_with_locals(
        py,
        pyo3_asyncio::tokio::get_current_locals(py)?,
        async move {
            let replaced_accent_phrases = method(rust_accent_phrases, speaker_id)
                .await
                .into_py_result()?;
            Python::with_gil(|py| {
                let replaced_accent_phrases = replaced_accent_phrases
                    .iter()
                    .map(move |accent_phrase| {
                        to_pydantic_dataclass(
                            accent_phrase,
                            py.import("voicevox_core")?.getattr("AccentPhrase")?,
                        )
                    })
                    .collect::<PyResult<Vec<_>>>()?;
                let replaced_accent_phrases = PyList::new(py, replaced_accent_phrases);
                Ok(replaced_accent_phrases.to_object(py))
            })
        },
    )
}
pub fn to_rust_uuid(ob: &PyAny) -> PyResult<Uuid> {
    let uuid = ob.getattr("hex")?.extract::<String>()?;
    uuid.parse().into_py_result()
}
pub fn to_py_uuid(py: Python, uuid: Uuid) -> PyResult<PyObject> {
    let uuid = uuid.hyphenated().to_string();
    let uuid = py.import("uuid")?.call_method1("UUID", (uuid,))?;
    Ok(uuid.to_object(py))
}
pub fn to_rust_user_dict_word(ob: &PyAny) -> PyResult<voicevox_core::UserDictWord> {
    voicevox_core::UserDictWord::new(
        ob.getattr("surface")?.extract()?,
        ob.getattr("pronunciation")?.extract()?,
        ob.getattr("accent_type")?.extract()?,
        to_rust_word_type(ob.getattr("word_type")?.extract()?)?,
        ob.getattr("priority")?.extract()?,
    )
    .into_py_result()
}
pub fn to_py_user_dict_word<'py>(
    py: Python<'py>,
    word: &voicevox_core::UserDictWord,
) -> PyResult<&'py PyAny> {
    let class = py
        .import("voicevox_core")?
        .getattr("UserDictWord")?
        .downcast()?;
    to_pydantic_dataclass(word, class)
}
pub fn to_rust_word_type(word_type: &PyAny) -> PyResult<UserDictWordType> {
    let name = word_type.getattr("name")?.extract::<String>()?;

    serde_json::from_value::<UserDictWordType>(json!(name)).into_py_result()
}

#[ext]
pub impl<T, E: Display> Result<T, E> {
    fn into_py_result(self) -> PyResult<T> {
        self.map_err(|e| VoicevoxError::new_err(e.to_string()))
    }
}
