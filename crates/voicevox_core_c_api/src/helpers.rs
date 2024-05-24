use easy_ext::ext;
use std::{error::Error as _, ffi::CStr, fmt::Debug, iter};
use uuid::Uuid;
use voicevox_core::{AudioQueryModel, UserDictWord, VoiceModelId};

use thiserror::Error;
use tracing::error;

use voicevox_core::AccentPhraseModel;

use crate::{
    result_code::VoicevoxResultCode, VoicevoxAccelerationMode, VoicevoxInitializeOptions,
    VoicevoxSynthesisOptions, VoicevoxTtsOptions, VoicevoxUserDictWord, VoicevoxUserDictWordType,
};

pub(crate) fn into_result_code_with_error(result: CApiResult<()>) -> VoicevoxResultCode {
    if let Err(err) = &result {
        display_error(err);
    }
    return into_result_code(result);

    fn display_error(err: &CApiError) {
        itertools::chain(
            [err.to_string()],
            iter::successors(err.source(), |&e| e.source()).map(|e| format!("Caused by: {e}")),
        )
        .for_each(|msg| error!("{msg}"));
    }

    fn into_result_code(result: CApiResult<()>) -> VoicevoxResultCode {
        use voicevox_core::ErrorKind::*;
        use CApiError::*;
        use VoicevoxResultCode::*;

        match result {
            Ok(()) => VOICEVOX_RESULT_OK,
            Err(RustApi(err)) => match err.kind() {
                NotLoadedOpenjtalkDict => VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR,
                GpuSupport => VOICEVOX_RESULT_GPU_SUPPORT_ERROR,
                OpenZipFile => VOICEVOX_RESULT_OPEN_ZIP_FILE_ERROR,
                ReadZipEntry => VOICEVOX_RESULT_READ_ZIP_ENTRY_ERROR,
                InvalidModelFormat => VOICEVOX_RESULT_INVALID_MODEL_HEADER_ERROR,
                ModelAlreadyLoaded => VOICEVOX_RESULT_MODEL_ALREADY_LOADED_ERROR,
                StyleAlreadyLoaded => VOICEVOX_RESULT_STYLE_ALREADY_LOADED_ERROR,
                InvalidModelData => VOICEVOX_RESULT_INVALID_MODEL_DATA_ERROR,
                GetSupportedDevices => VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR,
                StyleNotFound => VOICEVOX_RESULT_STYLE_NOT_FOUND_ERROR,
                ModelNotFound => VOICEVOX_RESULT_MODEL_NOT_FOUND_ERROR,
                InferenceFailed => VOICEVOX_RESULT_INFERENCE_ERROR,
                ExtractFullContextLabel => VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR,
                ParseKana => VOICEVOX_RESULT_PARSE_KANA_ERROR,
                LoadUserDict => VOICEVOX_RESULT_LOAD_USER_DICT_ERROR,
                SaveUserDict => VOICEVOX_RESULT_SAVE_USER_DICT_ERROR,
                WordNotFound => VOICEVOX_RESULT_USER_DICT_WORD_NOT_FOUND_ERROR,
                UseUserDict => VOICEVOX_RESULT_USE_USER_DICT_ERROR,
                InvalidWord => VOICEVOX_RESULT_INVALID_USER_DICT_WORD_ERROR,
            },
            Err(InvalidUtf8Input) => VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR,
            Err(InvalidAudioQuery(_)) => VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR,
            Err(InvalidAccentPhrase(_)) => VOICEVOX_RESULT_INVALID_ACCENT_PHRASE_ERROR,
            Err(InvalidUuid(_)) => VOICEVOX_RESULT_INVALID_UUID_ERROR,
        }
    }
}

pub(crate) type CApiResult<T> = std::result::Result<T, CApiError>;

#[derive(Error, Debug)]
pub(crate) enum CApiError {
    #[error("{0}")]
    RustApi(#[from] voicevox_core::Error),
    #[error("UTF-8として不正な入力です")]
    InvalidUtf8Input,
    #[error("無効なAudioQueryです: {0}")]
    InvalidAudioQuery(serde_json::Error),
    #[error("無効なAccentPhraseです: {0}")]
    InvalidAccentPhrase(serde_json::Error),
    #[error("無効なUUIDです: {0}")]
    InvalidUuid(uuid::Error),
}

pub(crate) fn audio_query_model_to_json(audio_query_model: &AudioQueryModel) -> String {
    serde_json::to_string(audio_query_model).expect("should be always valid")
}

pub(crate) fn accent_phrases_to_json(audio_query_model: &[AccentPhraseModel]) -> String {
    serde_json::to_string(audio_query_model).expect("should be always valid")
}

pub(crate) fn ensure_utf8(s: &CStr) -> CApiResult<&str> {
    s.to_str().map_err(|_| CApiError::InvalidUtf8Input)
}

impl From<VoicevoxSynthesisOptions> for voicevox_core::SynthesisOptions {
    fn from(options: VoicevoxSynthesisOptions) -> Self {
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

impl From<voicevox_core::AccelerationMode> for VoicevoxAccelerationMode {
    fn from(mode: voicevox_core::AccelerationMode) -> Self {
        use voicevox_core::AccelerationMode::*;
        match mode {
            Auto => Self::VOICEVOX_ACCELERATION_MODE_AUTO,
            Cpu => Self::VOICEVOX_ACCELERATION_MODE_CPU,
            Gpu => Self::VOICEVOX_ACCELERATION_MODE_GPU,
        }
    }
}

impl From<VoicevoxAccelerationMode> for voicevox_core::AccelerationMode {
    fn from(mode: VoicevoxAccelerationMode) -> Self {
        use VoicevoxAccelerationMode::*;
        match mode {
            VOICEVOX_ACCELERATION_MODE_AUTO => Self::Auto,
            VOICEVOX_ACCELERATION_MODE_CPU => Self::Cpu,
            VOICEVOX_ACCELERATION_MODE_GPU => Self::Gpu,
        }
    }
}

impl Default for VoicevoxInitializeOptions {
    fn default() -> Self {
        let options = voicevox_core::InitializeOptions::default();
        Self {
            acceleration_mode: options.acceleration_mode.into(),
            cpu_num_threads: options.cpu_num_threads,
        }
    }
}

impl From<VoicevoxInitializeOptions> for voicevox_core::InitializeOptions {
    fn from(value: VoicevoxInitializeOptions) -> Self {
        voicevox_core::InitializeOptions {
            acceleration_mode: value.acceleration_mode.into(),
            cpu_num_threads: value.cpu_num_threads,
        }
    }
}

impl From<voicevox_core::TtsOptions> for VoicevoxTtsOptions {
    fn from(options: voicevox_core::TtsOptions) -> Self {
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

impl From<VoicevoxTtsOptions> for voicevox_core::TtsOptions {
    fn from(options: VoicevoxTtsOptions) -> Self {
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

impl Default for VoicevoxSynthesisOptions {
    fn default() -> Self {
        let options = voicevox_core::TtsOptions::default();
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

#[ext(UuidBytesExt)]
pub(crate) impl uuid::Bytes {
    fn to_model_id(self) -> VoiceModelId {
        Uuid::from_bytes(self).into()
    }
}

impl VoicevoxUserDictWord {
    pub(crate) unsafe fn try_into_word(&self) -> CApiResult<voicevox_core::UserDictWord> {
        Ok(UserDictWord::new(
            ensure_utf8(CStr::from_ptr(self.surface))?,
            ensure_utf8(CStr::from_ptr(self.pronunciation))?.to_string(),
            self.accent_type,
            self.word_type.into(),
            self.priority,
        )?)
    }
}

impl From<VoicevoxUserDictWordType> for voicevox_core::UserDictWordType {
    fn from(value: VoicevoxUserDictWordType) -> Self {
        match value {
            VoicevoxUserDictWordType::VOICEVOX_USER_DICT_WORD_TYPE_PROPER_NOUN => Self::ProperNoun,
            VoicevoxUserDictWordType::VOICEVOX_USER_DICT_WORD_TYPE_COMMON_NOUN => Self::CommonNoun,
            VoicevoxUserDictWordType::VOICEVOX_USER_DICT_WORD_TYPE_VERB => Self::Verb,
            VoicevoxUserDictWordType::VOICEVOX_USER_DICT_WORD_TYPE_ADJECTIVE => Self::Adjective,
            VoicevoxUserDictWordType::VOICEVOX_USER_DICT_WORD_TYPE_SUFFIX => Self::Suffix,
        }
    }
}

impl From<voicevox_core::UserDictWordType> for VoicevoxUserDictWordType {
    fn from(value: voicevox_core::UserDictWordType) -> Self {
        match value {
            voicevox_core::UserDictWordType::ProperNoun => {
                Self::VOICEVOX_USER_DICT_WORD_TYPE_PROPER_NOUN
            }
            voicevox_core::UserDictWordType::CommonNoun => {
                Self::VOICEVOX_USER_DICT_WORD_TYPE_COMMON_NOUN
            }
            voicevox_core::UserDictWordType::Verb => Self::VOICEVOX_USER_DICT_WORD_TYPE_VERB,
            voicevox_core::UserDictWordType::Adjective => {
                Self::VOICEVOX_USER_DICT_WORD_TYPE_ADJECTIVE
            }
            voicevox_core::UserDictWordType::Suffix => Self::VOICEVOX_USER_DICT_WORD_TYPE_SUFFIX,
        }
    }
}
