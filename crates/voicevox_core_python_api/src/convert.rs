use std::{error::Error as _, iter, path::PathBuf};

use camino::Utf8PathBuf;
use derive_more::From;
use easy_ext::ext;
use heck::{ToLowerCamelCase as _, ToSnakeCase as _};
use num_bigint::BigInt;
use num_traits::Signed as _;
use pyo3::{
    Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyResult, Python,
    exceptions::{PyException, PyValueError},
    types::{
        IntoPyDict as _, PyAnyMethods as _, PyDict, PyDictMethods as _, PyList, PyListMethods as _,
        PyString, PyStringMethods as _,
    },
};
use ref_cast::RefCast;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use voicevox_core::{
    __internal::interop::{self, ToJsonValue as _, Validate},
    AccelerationMode, AccentPhrase, AudioQuery, FrameAudioQuery, OnExistingVoiceModelId,
    SupportedDevices, UserDictWord, UserDictWordPriority, VoiceModelMeta,
};

use crate::{
    _ReservedFields, AnalyzeTextError, GetSupportedDevicesError, GpuSupportError,
    IncompatibleQueriesError, InitInferenceRuntimeError, InvalidModelDataError,
    InvalidModelFormatError, InvalidQueryError, InvalidWordError, LoadUserDictError,
    ModelAlreadyLoadedError, ModelNotFoundError, NotLoadedOpenjtalkDictError, OpenZipFileError,
    ParseKanaError, ReadZipEntryError, RunModelError, SaveUserDictError, StyleAlreadyLoadedError,
    StyleNotFoundError, UseUserDictError, WordNotFoundError,
};

pub(crate) fn from_audio_feature_range_start(ob: &Bound<'_, PyAny>) -> PyResult<usize> {
    from_audio_feature_range_index(
        ob,
        |index| {
            format!(
                "argument 'start': must not be negative: {index} \
                 (note: instead, consider subtracting an offset from the \
                 'AudioFeature.frame_length')",
            )
        },
        |index| format!("argument 'start': cannot fit '{index}' into an index-sized integer"),
    )
}

pub(crate) fn from_audio_feature_range_stop(ob: &Bound<'_, PyAny>) -> PyResult<usize> {
    from_audio_feature_range_index(
        ob,
        |index| {
            format!(
                "argument 'stop': must not be negative: {index} \
                 (note: instead, consider subtracting an offset from the \
                 'AudioFeature.frame_length')",
            )
        },
        |index| {
            format!(
                "argument 'stop': cannot fit '{index}' into an index-sized integer \
                 (note: if you meant the end of the audio feature, consider using the \
                 'AudioFeature.frame_length')",
            )
        },
    )
}

fn from_audio_feature_range_index(
    ob: &Bound<'_, PyAny>,
    error_for_negative: fn(&BigInt) -> String,
    error_for_too_large: fn(&BigInt) -> String,
) -> PyResult<usize> {
    let index = &BigInt::extract(ob.as_borrowed())?;
    if index.is_negative() {
        return Err(PyValueError::new_err(error_for_negative(index)));
    }
    index
        .try_into()
        .map_err(|_| PyValueError::new_err(error_for_too_large(index)))
}

/// Rustの`[_; frame_length]`に対して`start..stop`が不適当であるなら[`PyValueError`]を返す。
///
/// # Motivation
///
/// Rust APIの`render`に不適当な範囲を渡すとパニックするため、渡す前に対処する必要がある。その方法は
///
/// 1. 範囲を補正してしまうか、
/// 2. `ValueError`として拒否するか
///
/// の２つ。
///
/// `list`や`NDArray`にならうならば1.だが、例えば同じPythonでもPillowではデフォルト値で埋められるし、OpenCVはエラーになる。
///
/// そのためVOICEVOX COREとしても言語間で挙動を統一するメリットの方を取ることとする。
///
/// <https://github.com/VOICEVOX/voicevox_core/pull/867#discussion_r1831401779>
pub(crate) fn error_for_audio_feature_range(
    frame_length: usize,
    start: usize,
    stop: usize,
) -> PyResult<()> {
    if vec![(); frame_length].get(start..stop).is_none() {
        return Err(PyValueError::new_err(format!(
            "Audio feature of length {frame_length} cannot accept '{start}:{stop}' as a valid \
             range: {reason}",
            reason = if start > frame_length {
                "'start' out of range for the audio feature"
            } else if stop > frame_length {
                "'stop' out of range for the audio feature \
                 (note: if you meant the end of the audio feature, consider using the \
                 'AudioFeature.frame_length')"
            } else {
                assert!(start > stop);
                "'start' exceeds 'stop'"
            }
        )));
    }
    Ok(())
}

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

pub(crate) fn from_on_existing_voice_model_id(
    ob: &Bound<'_, PyAny>,
) -> PyResult<OnExistingVoiceModelId> {
    match ob.extract::<&str>()? {
        "ERROR" => Ok(OnExistingVoiceModelId::Error),
        "RELOAD" => Ok(OnExistingVoiceModelId::Reload),
        "SKIP" => Ok(OnExistingVoiceModelId::Skip),
        on_existing => Err(PyValueError::new_err(format!(
            "`OnExistingVoiceModelId` should be one of {{ERROR, RELOAD, SKIP}}: {on_existing}",
            on_existing = PyString::new(ob.py(), on_existing).repr()?,
        ))),
    }
}

pub(crate) fn from_audio_query<T: HasCamelCaseFields>(ob: &Bound<'_, PyAny>) -> PyResult<T> {
    let py = ob.py();

    let fields = dataclasses_asdict(ob)?
        .iter()
        .map(|(key, value)| {
            let key = key.cast::<PyString>()?.to_str()?;
            let key = if T::SNAKE_CASE_FIELDS.contains(&key) {
                key.to_owned()
            } else {
                key.to_lower_camel_case()
            };
            Ok((key, value))
        })
        .collect::<PyResult<Vec<_>>>()?
        .into_py_dict(py)?;

    serde_pyobject::from_pyobject(fields).map_err(|serde_pyobject::Error(cause)| {
        let err = InvalidQueryError::new_err(T::validation_error_description());
        err.set_cause(py, Some(cause));
        err
    })
}

pub(crate) trait HasCamelCaseFields: Validate {
    const SNAKE_CASE_FIELDS: &[&str];
}

impl HasCamelCaseFields for AudioQuery {
    const SNAKE_CASE_FIELDS: &[&str] = &["accent_phrases"];
}

impl HasCamelCaseFields for FrameAudioQuery {
    const SNAKE_CASE_FIELDS: &[&str] = &[];
}

pub(crate) fn from_accent_phrases(ob: &Bound<'_, PyAny>) -> PyResult<Vec<AccentPhrase>> {
    ob.cast::<PyList>()?
        .iter()
        .map(|p| from_query_like_via_serde(&p))
        .collect()
}

pub(crate) fn from_utf8_path(ob: &Bound<'_, PyAny>) -> PyResult<Utf8PathBuf> {
    PathBuf::extract(ob.as_borrowed())?
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
                            .cast::<PyList>()?
                            .iter()
                            .map(|style| style_meta_cls.call((), Some(style.cast()?)))
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
                    .cast::<PyList>()?
                    .iter()
                    .map(|phrase| {
                        let phrase = phrase.cast::<PyDict>()?;
                        phrase.set_item(
                            "moras",
                            phrase
                                .get_item("moras")?
                                .expect("should be present")
                                .cast::<PyList>()?
                                .iter()
                                .map(|mora| mora_cls.call((), Some(mora.cast()?)))
                                .collect::<Result<Vec<_>, _>>()?,
                        )?;
                        accent_phrase_cls.call((), Some(phrase))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )?;
            for key in kwargs.keys().iter() {
                let key = key.cast::<PyString>()?.to_str()?;
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
                            .cast::<PyList>()?
                            .iter()
                            .map(|mora| mora_cls.call((), Some(mora.cast()?)))
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

impl RustData for FrameAudioQuery {
    type Target = PyAny;

    fn to_dataclass<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, Self::Target>> {
        let (frame_audio_query_cls, frame_phoneme_cls) = {
            let module = py.import("voicevox_core")?;
            (
                module.getattr("FrameAudioQuery")?,
                module.getattr("FramePhoneme")?,
            )
        };

        to_dataclass_via_serde(self, &frame_audio_query_cls, |kwargs| {
            kwargs.set_item(
                "phonemes",
                kwargs
                    .get_item("phonemes")?
                    .expect("should be present")
                    .cast::<PyList>()?
                    .iter()
                    .map(|frame_phoneme| frame_phoneme_cls.call((), Some(frame_phoneme.cast()?)))
                    .collect::<Result<Vec<_>, _>>()?,
            )?;
            for key in kwargs.keys().iter() {
                let key = key.cast::<PyString>()?.to_str()?;
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

pub(crate) fn from_query_like_via_serde<T: Validate>(instance: &Bound<'_, PyAny>) -> PyResult<T> {
    let py = instance.py();
    let fields = dataclasses_asdict(instance)?;
    serde_pyobject::from_pyobject(fields).map_err(|serde_pyobject::Error(cause)| {
        let err = InvalidQueryError::new_err(T::validation_error_description());
        err.set_cause(py, Some(cause));
        err
    })
}

fn dataclasses_asdict<'py>(instance: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
    let py = instance.py();
    let asdict = py.import("dataclasses")?.getattr("asdict")?;
    asdict.call1((instance,))?.cast_into().map_err(Into::into)
}

fn to_dataclass_via_serde<'py>(
    x: impl Serialize,
    class: &Bound<'py, PyAny>,
    modify: impl FnOnce(&Bound<'py, PyDict>) -> PyResult<()>,
) -> PyResult<Bound<'py, PyAny>> {
    let py = class.py();
    let kwargs = &serde_pyobject::to_pyobject(py, &x)?.cast_into::<PyDict>()?;
    modify(kwargs)?;
    class.call((), Some(kwargs))
}

pub(crate) fn to_rust_user_dict_word(
    ob: &Bound<'_, PyAny>,
) -> PyResult<voicevox_core::UserDictWord> {
    let priority = ob.getattr("priority")?.extract::<BigInt>()?;
    let priority = (&priority)
        .try_into()
        .ok()
        .and_then(|priority| UserDictWordPriority::new(priority).ok())
        .ok_or(interop::InvalidWordError::PriorityOutOfBounds {
            is_validation_of_whole_word: true,
            actual: priority,
        })
        .map_err(voicevox_core::Error::from)
        .into_py_result(ob.py())?;

    voicevox_core::UserDictWord::builder()
        .word_type(from_literal_choice(ob.getattr("word_type")?.extract()?)?)
        .priority(priority)
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
            kwargs.set_item("priority", word.priority().get())?;
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
                ErrorKind::InvalidQuery => InvalidQueryError::new_err(msg),
                ErrorKind::IncompatibleQueries => IncompatibleQueriesError::new_err(msg),
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
                let kwargs = serde_pyobject::to_pyobject(py, &self)?.cast_into::<PyDict>()?;
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
impl<T> serde_json::Result<T> {
    fn into_py_value_result(self) -> PyResult<T> {
        self.map_err(|e| PyValueError::new_err(e.to_string()))
    }
}
