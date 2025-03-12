use std::{error::Error as _, iter, path::PathBuf};

use camino::Utf8PathBuf;
use derive_more::From;
use easy_ext::ext;
use heck::{ToLowerCamelCase as _, ToSnakeCase as _};
use pyo3::{
    exceptions::{PyException, PyValueError},
    types::{
        IntoPyDict as _, PyAnyMethods as _, PyDict, PyDictMethods as _, PyList, PyListMethods as _,
        PyString, PyStringMethods as _,
    },
    Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python,
};
use ref_cast::RefCast;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use uuid::Uuid;
use voicevox_core::{
    AccelerationMode, AccentPhrase, AudioQuery, SupportedDevices, UserDictWord, VoiceModelMeta,
    __internal::interop::ToJsonValue as _,
};

use crate::{
    AnalyzeTextError, GetSupportedDevicesError, GpuSupportError, InitInferenceRuntimeError,
    InvalidModelDataError, InvalidModelFormatError, InvalidWordError, LoadUserDictError,
    ModelAlreadyLoadedError, ModelNotFoundError, NotLoadedOpenjtalkDictError, OpenZipFileError,
    ParseKanaError, ReadZipEntryError, RunModelError, SaveUserDictError, StyleAlreadyLoadedError,
    StyleNotFoundError, UseUserDictError, WordNotFoundError, _ReservedFields,
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

pub(crate) fn from_audio_query(ob: &Bound<'_, PyAny>) -> PyResult<AudioQuery> {
    let py = ob.py();

    let fields = dataclasses_asdict(ob)?
        .iter()
        .map(|(key, value)| {
            let key = match key.downcast::<PyString>()?.to_str()? {
                "accent_phrases" => "accent_phrases".to_owned(),
                key => key.to_lower_camel_case(),
            };
            Ok((key, value))
        })
        .collect::<PyResult<Vec<_>>>()?
        .into_py_dict(py)?;

    serde_pyobject::from_pyobject(fields).map_err(Into::into)
}

pub(crate) fn from_accent_phrases(ob: &Bound<'_, PyAny>) -> PyResult<Vec<AccentPhrase>> {
    ob.downcast::<PyList>()?
        .iter()
        .map(|p| from_dataclass_via_serde(&p))
        .collect()
}

pub(crate) fn from_utf8_path(ob: &Bound<'_, PyAny>) -> PyResult<Utf8PathBuf> {
    PathBuf::extract_bound(ob)?
        .into_os_string()
        .into_string()
        .map(Utf8PathBuf::from)
        .map_err(|s| PyValueError::new_err(format!("{s:?} cannot be encoded to UTF-8")))
}

/// Pythonのデータクラスもしくはデータクラスのリストへの変換。
#[derive(From, RefCast)]
#[repr(transparent)]
pub(crate) struct ToDataclass<T>(T);

impl<'py, T: RustData> IntoPyObject<'py> for ToDataclass<T> {
    type Target = T::Target;
    type Output = Bound<'py, T::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.0.to_dataclass(py)
    }
}

impl<'py, T: RustData> IntoPyObject<'py> for &'_ ToDataclass<T> {
    type Target = T::Target;
    type Output = Bound<'py, T::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        self.0.to_dataclass(py)
    }
}

pub(crate) trait RustData {
    type Target;
    fn to_dataclass<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, Self::Target>>;
}

impl RustData for SupportedDevices {
    type Target = PyAny;

    fn to_dataclass<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, Self::Target>> {
        self.to_py(py)
    }
}

impl RustData for VoiceModelMeta {
    type Target = PyList;

    fn to_dataclass<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, Self::Target>> {
        let (character_meta_cls, style_meta_cls) = {
            let module = py.import("voicevox_core")?;
            (
                module.getattr("CharacterMeta")?,
                module.getattr("StyleMeta")?,
            )
        };

        let metas = self
            .iter()
            .map(|meta| {
                to_dataclass_via_serde(meta, &character_meta_cls, |kwargs| {
                    kwargs.set_item(
                        "styles",
                        kwargs
                            .get_item("styles")?
                            .expect("should be present")
                            .downcast::<PyList>()?
                            .iter()
                            .map(|style| style_meta_cls.call((), Some(style.downcast()?)))
                            .collect::<Result<Vec<_>, _>>()?,
                    )
                })
            })
            .collect::<PyResult<Vec<_>>>()?;
        PyList::new(py, metas)
    }
}

impl RustData for AudioQuery {
    type Target = PyAny;

    fn to_dataclass<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, Self::Target>> {
        let (audio_query_cls, accent_phrase_cls, mora_cls) = {
            let module = py.import("voicevox_core")?;
            (
                module.getattr("AudioQuery")?,
                module.getattr("AccentPhrase")?,
                module.getattr("Mora")?,
            )
        };

        to_dataclass_via_serde(self, &audio_query_cls, |kwargs| {
            kwargs.set_item(
                "accent_phrases",
                kwargs
                    .get_item("accent_phrases")?
                    .expect("should be present")
                    .downcast::<PyList>()?
                    .iter()
                    .map(|phrase| {
                        let phrase = phrase.downcast::<PyDict>()?;
                        phrase.set_item(
                            "moras",
                            phrase
                                .get_item("moras")?
                                .expect("should be present")
                                .downcast::<PyList>()?
                                .iter()
                                .map(|mora| mora_cls.call((), Some(mora.downcast()?)))
                                .collect::<Result<Vec<_>, _>>()?,
                        )?;
                        accent_phrase_cls.call((), Some(phrase))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )?;
            for key in kwargs.keys().iter() {
                let key = key.downcast::<PyString>()?.to_str()?;
                let key_rename = key.to_snake_case();
                if key_rename != key {
                    let val = kwargs.get_item(key)?.expect("should be present");
                    kwargs.set_item(key_rename, val)?;
                    kwargs.del_item(key)?;
                }
            }
            Ok(())
        })
    }
}

impl RustData for Vec<AccentPhrase> {
    type Target = PyList;

    fn to_dataclass<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, Self::Target>> {
        let (accent_phrase_cls, mora_cls) = {
            let module = py.import("voicevox_core")?;
            (module.getattr("AccentPhrase")?, module.getattr("Mora")?)
        };

        let phrases = self
            .iter()
            .map(|phrase| {
                to_dataclass_via_serde(phrase, &accent_phrase_cls, |kwargs| {
                    kwargs.set_item(
                        "moras",
                        kwargs
                            .get_item("moras")?
                            .expect("should be present")
                            .downcast::<PyList>()?
                            .iter()
                            .map(|mora| mora_cls.call((), Some(mora.downcast()?)))
                            .collect::<Result<Vec<_>, _>>()?,
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        PyList::new(py, phrases)
    }
}

impl RustData for UserDictWord {
    type Target = PyAny;

    fn to_dataclass<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, Self::Target>> {
        to_py_user_dict_word(py, self)
    }
}

#[derive(From)]
pub(crate) struct ToPyUuid(pub(crate) Uuid);

impl<'py> IntoPyObject<'py> for ToPyUuid {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        to_py_uuid(py, self.0)
    }
}

fn from_dataclass_via_serde<T: DeserializeOwned>(instance: &Bound<'_, PyAny>) -> PyResult<T> {
    let fields = dataclasses_asdict(instance)?;
    serde_pyobject::from_pyobject(fields).map_err(Into::into)
}

fn dataclasses_asdict<'py>(instance: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
    let py = instance.py();
    let asdict = py.import("dataclasses")?.getattr("asdict")?;
    asdict
        .call1((instance,))?
        .downcast_into()
        .map_err(Into::into)
}

fn to_dataclass_via_serde<'py>(
    x: impl Serialize,
    class: &Bound<'py, PyAny>,
    modify: impl FnOnce(&Bound<'py, PyDict>) -> PyResult<()>,
) -> PyResult<Bound<'py, PyAny>> {
    let py = class.py();
    let kwargs = &serde_pyobject::to_pyobject(py, &x)?.downcast_into::<PyDict>()?;
    modify(kwargs)?;
    class.call((), Some(kwargs))
}

pub(crate) fn to_rust_uuid(ob: &Bound<'_, PyAny>) -> PyResult<Uuid> {
    ob.getattr("hex")?
        .extract::<&str>()?
        .parse::<Uuid>()
        .into_py_value_result()
}
fn to_py_uuid(py: Python<'_>, uuid: Uuid) -> PyResult<Bound<'_, PyAny>> {
    let uuid = uuid.hyphenated().to_string();
    py.import("uuid")?.call_method1("UUID", (uuid,))
}
pub(crate) fn to_rust_user_dict_word(
    ob: &Bound<'_, PyAny>,
) -> PyResult<voicevox_core::UserDictWord> {
    voicevox_core::UserDictWord::builder()
        .word_type(from_literal_choice(ob.getattr("word_type")?.extract()?)?)
        .priority(ob.getattr("priority")?.extract()?)
        .build(
            ob.getattr("surface")?.extract()?,
            ob.getattr("pronunciation")?.extract()?,
            ob.getattr("accent_type")?.extract()?,
        )
        .into_py_result(ob.py())
}
fn to_py_user_dict_word<'py>(
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

#[ext]
impl SupportedDevices {
    fn to_py(self, py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        assert!(match self.to_json_value() {
            serde_json::Value::Object(o) => o.len() == 3, // `cpu`, `cuda`, `dml`
            _ => false,
        });

        let cls = py.import("voicevox_core")?.getattr("SupportedDevices")?;
        cls.call(
            (),
            Some(&{
                let kwargs = serde_pyobject::to_pyobject(py, &self)?.downcast_into::<PyDict>()?;
                kwargs.set_item("_reserved", _ReservedFields)?;
                kwargs
            }),
        )
    }
}

#[ext(AudioQueryExt)]
impl AudioQuery
where
    Self: Sized,
{
    pub(crate) fn from_json(json: &str) -> PyResult<Self> {
        serde_json::from_str(json).into_py_value_result()
    }

    pub(crate) fn to_json(&self) -> String {
        serde_json::to_string(self).expect("should not fail")
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
