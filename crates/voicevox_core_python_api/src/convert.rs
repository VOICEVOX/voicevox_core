use std::{error::Error as _, future::Future, iter, panic, path::PathBuf};

use camino::Utf8PathBuf;
use easy_ext::ext;
use pyo3::{
    exceptions::{PyException, PyRuntimeError, PyValueError},
    types::{IntoPyDict as _, PyList},
    FromPyObject as _, IntoPy, PyAny, PyObject, PyResult, Python, ToPyObject,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use uuid::Uuid;
use voicevox_core::{AccelerationMode, AccentPhrase, StyleId, UserDictWordType, VoiceModelMeta};

use crate::{
    AnalyzeTextError, GetSupportedDevicesError, GpuSupportError, InitInferenceRuntimeError,
    InvalidModelDataError, InvalidModelFormatError, InvalidWordError, LoadUserDictError,
    ModelAlreadyLoadedError, ModelNotFoundError, NotLoadedOpenjtalkDictError, OpenZipFileError,
    ParseKanaError, ReadZipEntryError, RunModelError, SaveUserDictError, StyleAlreadyLoadedError,
    StyleNotFoundError, UseUserDictError, WordNotFoundError,
};

pub(crate) fn from_acceleration_mode(ob: &PyAny) -> PyResult<AccelerationMode> {
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

pub(crate) fn from_utf8_path(ob: &PyAny) -> PyResult<Utf8PathBuf> {
    PathBuf::extract(ob)?
        .into_os_string()
        .into_string()
        .map(Utf8PathBuf::from)
        .map_err(|s| PyValueError::new_err(format!("{s:?} cannot be encoded to UTF-8")))
}

pub(crate) fn from_dataclass<T: DeserializeOwned>(ob: &PyAny) -> PyResult<T> {
    let py = ob.py();

    let ob = py.import("dataclasses")?.call_method1("asdict", (ob,))?;
    let json = &py
        .import("json")?
        .call_method1("dumps", (ob,))?
        .extract::<String>()?;
    serde_json::from_str(json).into_py_value_result()
}

pub(crate) fn to_pydantic_voice_model_meta<'py>(
    metas: &VoiceModelMeta,
    py: Python<'py>,
) -> PyResult<&'py PyList> {
    let class = py
        .import("voicevox_core")?
        .getattr("SpeakerMeta")?
        .downcast()?;

    let metas = metas
        .iter()
        .map(|m| to_pydantic_dataclass(m, class))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyList::new(py, metas))
}

pub(crate) fn to_pydantic_dataclass(x: impl Serialize, class: &PyAny) -> PyResult<&PyAny> {
    let py = class.py();

    let x = serde_json::to_string(&x).into_py_value_result()?;
    let x = py.import("json")?.call_method1("loads", (x,))?.downcast()?;
    class.call((), Some(x))
}

pub(crate) fn blocking_modify_accent_phrases<'py>(
    accent_phrases: &'py PyList,
    speaker_id: StyleId,
    py: Python<'py>,
    method: impl FnOnce(Vec<AccentPhrase>, StyleId) -> voicevox_core::Result<Vec<AccentPhrase>>,
) -> PyResult<Vec<&'py PyAny>> {
    let rust_accent_phrases = accent_phrases
        .iter()
        .map(from_dataclass)
        .collect::<PyResult<Vec<AccentPhrase>>>()?;

    method(rust_accent_phrases, speaker_id)
        .into_py_result(py)?
        .iter()
        .map(move |accent_phrase| {
            to_pydantic_dataclass(
                accent_phrase,
                py.import("voicevox_core")?.getattr("AccentPhrase")?,
            )
        })
        .collect()
}

pub(crate) fn async_modify_accent_phrases<'py, Fun, Fut>(
    accent_phrases: &'py PyList,
    speaker_id: StyleId,
    py: Python<'py>,
    method: Fun,
) -> PyResult<&'py PyAny>
where
    Fun: FnOnce(Vec<AccentPhrase>, StyleId) -> Fut + Send + 'static,
    Fut: Future<Output = PyResult<Vec<AccentPhrase>>> + Send + 'static,
{
    let rust_accent_phrases = accent_phrases
        .iter()
        .map(from_dataclass)
        .collect::<PyResult<Vec<AccentPhrase>>>()?;
    pyo3_asyncio::tokio::future_into_py_with_locals(
        py,
        pyo3_asyncio::tokio::get_current_locals(py)?,
        async move {
            let replaced_accent_phrases = method(rust_accent_phrases, speaker_id).await?;
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

pub(crate) fn to_rust_uuid(ob: &PyAny) -> PyResult<Uuid> {
    let uuid = ob.getattr("hex")?.extract::<String>()?;
    uuid.parse::<Uuid>().into_py_value_result()
}
pub(crate) fn to_py_uuid(py: Python<'_>, uuid: Uuid) -> PyResult<PyObject> {
    let uuid = uuid.hyphenated().to_string();
    let uuid = py.import("uuid")?.call_method1("UUID", (uuid,))?;
    Ok(uuid.to_object(py))
}
pub(crate) fn to_rust_user_dict_word(ob: &PyAny) -> PyResult<voicevox_core::UserDictWord> {
    voicevox_core::UserDictWord::new(
        ob.getattr("surface")?.extract()?,
        ob.getattr("pronunciation")?.extract()?,
        ob.getattr("accent_type")?.extract()?,
        to_rust_word_type(ob.getattr("word_type")?.extract()?)?,
        ob.getattr("priority")?.extract()?,
    )
    .into_py_result(ob.py())
}
pub(crate) fn to_py_user_dict_word<'py>(
    py: Python<'py>,
    word: &voicevox_core::UserDictWord,
) -> PyResult<&'py PyAny> {
    let class = py
        .import("voicevox_core")?
        .getattr("UserDictWord")?
        .downcast()?;
    to_pydantic_dataclass(word, class)
}
pub(crate) fn to_rust_word_type(word_type: &PyAny) -> PyResult<UserDictWordType> {
    let name = word_type.getattr("name")?.extract::<String>()?;

    serde_json::from_value::<UserDictWordType>(json!(name)).into_py_value_result()
}

/// おおよそ以下のコードにおける`f(x)`のようなものを得る。
///
/// ```py
/// async def f(x_):
///     return x_
///
/// return f(x)
/// ```
pub(crate) fn ready(x: impl IntoPy<PyObject>, py: Python<'_>) -> PyResult<&PyAny> {
    // ```py
    // from asyncio import Future
    //
    // running_loop = asyncio.get_running_loop()
    // fut = Future(loop=running_loop)
    // fut.set_result(x)
    // return fut
    // ```

    let asyncio_future = py.import("asyncio")?.getattr("Future")?;

    let running_loop = pyo3_asyncio::get_running_loop(py)?;
    let fut = asyncio_future.call((), Some([("loop", running_loop)].into_py_dict(py)))?;
    fut.call_method1("set_result", (x,))?;
    Ok(fut)
}

pub(crate) async fn run_in_executor<F, R>(f: F) -> PyResult<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| match e.try_into_panic() {
            Ok(p) => panic::resume_unwind(p),
            Err(e) => PyRuntimeError::new_err(e.to_string()),
        })
}

#[ext(VoicevoxCoreResultExt)]
pub(crate) impl<T> voicevox_core::Result<T> {
    fn into_py_result(self, py: Python<'_>) -> PyResult<T> {
        use voicevox_core::ErrorKind;

        self.map_err(|err| {
            let msg = err.to_string();
            let top = match err.kind() {
                ErrorKind::NotLoadedOpenjtalkDict => NotLoadedOpenjtalkDictError::new_err(msg),
                ErrorKind::GpuSupport => GpuSupportError::new_err(msg),
                ErrorKind::InitInferenceRuntime => InitInferenceRuntimeError::new_err(msg),
                ErrorKind::OpenZipFile => OpenZipFileError::new_err(msg),
                ErrorKind::ReadZipEntry => ReadZipEntryError::new_err(msg),
                ErrorKind::ModelAlreadyLoaded => ModelAlreadyLoadedError::new_err(msg),
                ErrorKind::StyleAlreadyLoaded => StyleAlreadyLoadedError::new_err(msg),
                ErrorKind::InvalidModelFormat => InvalidModelFormatError::new_err(msg),
                ErrorKind::InvalidModelData => InvalidModelDataError::new_err(msg),
                ErrorKind::GetSupportedDevices => GetSupportedDevicesError::new_err(msg),
                ErrorKind::StyleNotFound => StyleNotFoundError::new_err(msg),
                ErrorKind::ModelNotFound => ModelNotFoundError::new_err(msg),
                ErrorKind::RunModel => RunModelError::new_err(msg),
                ErrorKind::AnalyzeText => AnalyzeTextError::new_err(msg),
                ErrorKind::ParseKana => ParseKanaError::new_err(msg),
                ErrorKind::LoadUserDict => LoadUserDictError::new_err(msg),
                ErrorKind::SaveUserDict => SaveUserDictError::new_err(msg),
                ErrorKind::WordNotFound => WordNotFoundError::new_err(msg),
                ErrorKind::UseUserDict => UseUserDictError::new_err(msg),
                ErrorKind::InvalidWord => InvalidWordError::new_err(msg),
            };

            [top]
                .into_iter()
                .chain(
                    iter::successors(err.source(), |&source| source.source())
                        .map(|source| PyException::new_err(source.to_string())),
                )
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .reduce(|prev, source| {
                    source.set_cause(py, Some(prev));
                    source
                })
                .expect("should not be empty")
        })
    }
}

#[ext]
impl<T> std::result::Result<T, uuid::Error> {
    fn into_py_value_result(self) -> PyResult<T> {
        self.map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

#[ext]
impl<T> serde_json::Result<T> {
    fn into_py_value_result(self) -> PyResult<T> {
        self.map_err(|e| PyValueError::new_err(e.to_string()))
    }
}
