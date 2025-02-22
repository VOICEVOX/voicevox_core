use std::{error::Error as _, future::Future, iter, path::PathBuf};

use camino::Utf8PathBuf;
use duplicate::duplicate_item;
use easy_ext::ext;
use pyo3::{
    exceptions::{PyException, PyValueError},
    types::{
        IntoPyDict as _, PyAnyMethods as _, PyBytes, PyBytesMethods as _, PyDict,
        PyDictMethods as _, PyList, PyListMethods as _, PyString,
    },
    Bound, FromPyObject as _, IntoPyObject, Py, PyAny, PyResult, Python,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use uuid::Uuid;
use voicevox_core::{
    AccelerationMode, AccentPhrase, AudioQuery, StyleId, SupportedDevices, VoiceModelMeta,
    __internal::interop::ToJsonValue as _,
};

use crate::{
    AnalyzeTextError, GetSupportedDevicesError, GpuSupportError, InitInferenceRuntimeError,
    InvalidModelDataError, InvalidModelFormatError, InvalidWordError, LoadUserDictError,
    ModelAlreadyLoadedError, ModelNotFoundError, NotLoadedOpenjtalkDictError, OpenZipFileError,
    ParseKanaError, ReadZipEntryError, RunModelError, SaveUserDictError, StyleAlreadyLoadedError,
    StyleNotFoundError, UseUserDictError, WordNotFoundError,
};

pub(crate) fn from_acceleration_mode(ob: &Bound<'_, PyAny>) -> PyResult<AccelerationMode> {
    match ob.extract::<&str>()? {
        "AUTO" => Ok(AccelerationMode::Auto),
        "CPU" => Ok(AccelerationMode::Cpu),
        "GPU" => Ok(AccelerationMode::Gpu),
        mode => Err(PyValueError::new_err(format!(
            "`AccelerationMode` should be one of {{AUTO, CPU, GPU}}: {mode}",
            mode = PyString::new(ob.py(), mode).repr()?,
        ))),
    }
}

pub(crate) fn from_utf8_path(ob: &Bound<'_, PyAny>) -> PyResult<Utf8PathBuf> {
    PathBuf::extract_bound(ob)?
        .into_os_string()
        .into_string()
        .map(Utf8PathBuf::from)
        .map_err(|s| PyValueError::new_err(format!("{s:?} cannot be encoded to UTF-8")))
}

pub(crate) trait HasClass: DeserializeOwned {
    fn cls(py: Python<'_>) -> PyResult<Bound<'_, PyAny>>;
}

#[duplicate_item(
    T;
    [ AudioQuery ];
    [ AccentPhrase ];
)]
impl HasClass for T {
    fn cls(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        py.import("voicevox_core")?.getattr(stringify!(T))
    }
}

pub(crate) fn from_dataclass<T: HasClass>(ob: &Bound<'_, PyAny>) -> PyResult<T> {
    let py = ob.py();

    let type_adapter = py.import("pydantic")?.getattr("TypeAdapter")?;
    let json = type_adapter.call1((T::cls(py)?,))?.call_method(
        "dump_json",
        (ob,),
        Some(&[("by_alias", true)].into_py_dict(py)?),
    )?;
    serde_json::from_slice(json.downcast::<PyBytes>()?.as_bytes()).into_py_value_result()
}

pub(crate) fn to_pydantic_voice_model_meta<'py>(
    metas: &VoiceModelMeta,
    py: Python<'py>,
) -> PyResult<Bound<'py, PyList>> {
    let class = &py.import("voicevox_core")?.getattr("CharacterMeta")?;

    let metas = metas
        .iter()
        .map(|m| to_pydantic_dataclass(m, class))
        .collect::<PyResult<Vec<_>>>()?;
    PyList::new(py, metas)
}

pub(crate) fn to_pydantic_dataclass<'py>(
    x: impl Serialize,
    class: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    let py = class.py();

    let x = serde_json::to_string(&x).into_py_value_result()?;
    let x = py.import("json")?.call_method1("loads", (x,))?;
    let x = x.downcast()?;
    class.call((), Some(x))
}

pub(crate) fn blocking_modify_accent_phrases<'py>(
    accent_phrases: &Bound<'py, PyList>,
    speaker_id: StyleId,
    py: Python<'py>,
    method: impl FnOnce(Vec<AccentPhrase>, StyleId) -> voicevox_core::Result<Vec<AccentPhrase>>,
) -> PyResult<Vec<Bound<'py, PyAny>>> {
    let rust_accent_phrases = accent_phrases
        .iter()
        .map(|x| from_dataclass(&x))
        .collect::<PyResult<Vec<AccentPhrase>>>()?;

    method(rust_accent_phrases, speaker_id)
        .into_py_result(py)?
        .iter()
        .map(move |accent_phrase| {
            to_pydantic_dataclass(
                accent_phrase,
                &py.import("voicevox_core")?.getattr("AccentPhrase")?,
            )
        })
        .collect()
}

pub(crate) async fn async_modify_accent_phrases<Fun, Fut>(
    accent_phrases: Py<PyList>,
    speaker_id: StyleId,
    method: Fun,
) -> PyResult<Py<PyList>>
where
    Fun: FnOnce(Vec<AccentPhrase>, StyleId) -> Fut + Send + 'static,
    Fut: Future<Output = PyResult<Vec<AccentPhrase>>> + Send + 'static,
{
    let rust_accent_phrases = Python::with_gil(|py| {
        accent_phrases
            .into_bound(py)
            .iter()
            .map(|x| from_dataclass(&x))
            .collect::<PyResult<Vec<AccentPhrase>>>()
    })?;
    let replaced_accent_phrases = method(rust_accent_phrases, speaker_id).await?;
    Python::with_gil(|py| {
        let replaced_accent_phrases = replaced_accent_phrases
            .iter()
            .map(move |accent_phrase| {
                to_pydantic_dataclass(
                    accent_phrase,
                    &py.import("voicevox_core")?.getattr("AccentPhrase")?,
                )
            })
            .collect::<PyResult<Vec<_>>>()?;
        PyList::new(py, replaced_accent_phrases).map(Into::into)
    })
}

pub(crate) fn to_rust_uuid(ob: &Bound<'_, PyAny>) -> PyResult<Uuid> {
    ob.getattr("hex")?
        .extract::<&str>()?
        .parse::<Uuid>()
        .into_py_value_result()
}
pub(crate) fn to_py_uuid(py: Python<'_>, uuid: Uuid) -> PyResult<Bound<'_, PyAny>> {
    let uuid = uuid.hyphenated().to_string();
    py.import("uuid")?.call_method1("UUID", (uuid,))
}
pub(crate) fn to_rust_user_dict_word(
    ob: &Bound<'_, PyAny>,
) -> PyResult<voicevox_core::UserDictWord> {
    voicevox_core::UserDictWord::new(
        ob.getattr("surface")?.extract()?,
        ob.getattr("pronunciation")?.extract()?,
        ob.getattr("accent_type")?.extract()?,
        from_literal_choice(ob.getattr("word_type")?.extract()?)?,
        ob.getattr("priority")?.extract()?,
    )
    .into_py_result(ob.py())
}
pub(crate) fn to_py_user_dict_word<'py>(
    py: Python<'py>,
    word: &voicevox_core::UserDictWord,
) -> PyResult<Bound<'py, PyAny>> {
    let class = py.import("voicevox_core")?.getattr("UserDictWord")?;

    class.call(
        (),
        Some(&{
            let kwargs = PyDict::new(py);
            kwargs.set_item("surface", word.surface())?;
            kwargs.set_item("pronunciation", word.pronunciation())?;
            kwargs.set_item("accent_type", word.accent_type())?;
            kwargs.set_item(
                "word_type",
                serde_json::to_value(word.word_type())
                    .expect("should success")
                    .as_str()
                    .expect("should be a string"),
            )?;
            kwargs.set_item("priority", word.priority())?;
            kwargs
        }),
    )
}
fn from_literal_choice<T: DeserializeOwned>(s: &str) -> PyResult<T> {
    serde_json::from_value::<T>(json!(s)).into_py_value_result()
}

/// おおよそ以下のコードにおける`f(x)`のようなものを得る。
///
/// ```py
/// async def f(x_):
///     return x_
///
/// return f(x)
/// ```
pub(crate) fn ready<'py>(
    x: impl IntoPyObject<'py>,
    py: Python<'py>,
) -> PyResult<Bound<'py, PyAny>> {
    // ```py
    // import asyncio
    // from asyncio import Future
    //
    // running_loop = asyncio.get_running_loop()
    // fut = Future(loop=running_loop)
    // fut.set_result(x)
    // return fut
    // ```

    let asyncio = py.import("asyncio")?;
    let asyncio_future = asyncio.getattr("Future")?;

    let running_loop = asyncio.call_method0("get_running_loop")?;
    let fut = asyncio_future.call((), Some(&[("loop", running_loop)].into_py_dict(py)?))?;
    fut.call_method1("set_result", (x,))?;
    Ok(fut)
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
                ErrorKind::__NonExhaustive => unreachable!(),
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

#[ext(SupportedDevicesExt)]
impl SupportedDevices {
    pub(crate) fn to_py(self, py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        assert!(match self.to_json_value() {
            serde_json::Value::Object(o) => o.len() == 3, // `cpu`, `cuda`, `dml`
            _ => false,
        });

        let cls = py.import("voicevox_core")?.getattr("SupportedDevices")?;
        cls.call(
            ("I AM FROM PYO3",),
            Some(&[("cpu", self.cpu), ("cuda", self.cuda), ("dml", self.dml)].into_py_dict(py)?),
        )
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
