use std::fmt::Debug;
use voicevox_core::UserDictWord;

use const_default::ConstDefault;
use thiserror::Error;

use super::*;
use voicevox_core::AccentPhraseModel;

pub(crate) fn into_result_code_with_error(result: CApiResult<()>) -> VoicevoxResultCode {
    if let Err(err) = &result {
        display_error(err);
    }
    return into_result_code(result);

    fn display_error(err: &CApiError) {
        eprintln!("Error(Display): {err}");
        eprintln!("Error(Debug): {err:#?}");
    }

    fn into_result_code(result: CApiResult<()>) -> VoicevoxResultCode {
        use voicevox_core::{result_code::VoicevoxResultCode::*, Error::*};
        use CApiError::*;

        match result {
            Ok(()) => VOICEVOX_RESULT_OK,
            Err(RustApi(NotLoadedOpenjtalkDict)) => VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR,
            Err(RustApi(GpuSupport)) => VOICEVOX_RESULT_GPU_SUPPORT_ERROR,
            Err(RustApi(LoadModel { .. })) => VOICEVOX_RESULT_LOAD_MODEL_ERROR,
            Err(RustApi(LoadMetas(_))) => VOICEVOX_RESULT_LOAD_METAS_ERROR,
            Err(RustApi(GetSupportedDevices(_))) => VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR,
            Err(RustApi(InvalidStyleId { .. })) => VOICEVOX_RESULT_INVALID_STYLE_ID_ERROR,
            Err(RustApi(InvalidModelIndex { .. })) => VOICEVOX_RESULT_INVALID_MODEL_INDEX_ERROR,
            Err(RustApi(InferenceFailed)) => VOICEVOX_RESULT_INFERENCE_ERROR,
            Err(RustApi(ExtractFullContextLabel(_))) => {
                VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR
            }
            Err(RustApi(UnloadedModel { .. })) => VOICEVOX_UNLOADED_MODEL_ERROR,
            Err(RustApi(AlreadyLoadedModel { .. })) => VOICEVOX_ALREADY_LOADED_MODEL_ERROR,
            Err(RustApi(OpenFile { .. })) => VOICEVOX_OPEN_FILE_ERROR,
            Err(RustApi(VvmRead { .. })) => VOICEVOX_VVM_MODEL_READ_ERROR,
            Err(RustApi(ParseKana(_))) => VOICEVOX_RESULT_PARSE_KANA_ERROR,
            Err(RustApi(UserDictLoad(_))) => VOICEVOX_OPEN_JTALK_LOAD_USER_DICT_ERROR,
            Err(RustApi(UserDictSave(_))) => VOICEVOX_USER_DICT_SAVE_ERROR,
            Err(RustApi(WordNotFound(_))) => VOICEVOX_USER_DICT_WORD_NOT_FOUND_ERROR,
            Err(RustApi(OpenjtalkLoadUserDict(_))) => VOICEVOX_OPEN_JTALK_LOAD_USER_DICT_ERROR,
            Err(RustApi(InvalidWord(_))) => VOICEVOX_USER_DICT_INVALID_WORD_ERROR,
            Err(InvalidUtf8Input) => VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR,
            Err(InvalidAudioQuery(_)) => VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR,
            Err(InvalidAccentPhrase(_)) => VOICEVOX_RESULT_INVALID_ACCENT_PHRASE_ERROR,
        }
    }
}

type CApiResult<T> = std::result::Result<T, CApiError>;

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

impl ConstDefault for VoicevoxAudioQueryOptions {
    const DEFAULT: Self = {
        let options = voicevox_core::AudioQueryOptions::DEFAULT;
        Self { kana: options.kana }
    };
}
impl From<VoicevoxAudioQueryOptions> for voicevox_core::AudioQueryOptions {
    fn from(options: VoicevoxAudioQueryOptions) -> Self {
        Self { kana: options.kana }
    }
}

impl ConstDefault for VoicevoxAccentPhrasesOptions {
    const DEFAULT: Self = {
        let options = voicevox_core::AccentPhrasesOptions::DEFAULT;
        Self { kana: options.kana }
    };
}
impl From<VoicevoxAccentPhrasesOptions> for voicevox_core::AccentPhrasesOptions {
    fn from(options: VoicevoxAccentPhrasesOptions) -> Self {
        Self { kana: options.kana }
    }
}

impl From<VoicevoxSynthesisOptions> for voicevox_core::SynthesisOptions {
    fn from(options: VoicevoxSynthesisOptions) -> Self {
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

impl VoicevoxAccelerationMode {
    const fn from_rust(mode: voicevox_core::AccelerationMode) -> Self {
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

impl ConstDefault for VoicevoxInitializeOptions {
    const DEFAULT: Self = {
        let options = voicevox_core::InitializeOptions::DEFAULT;
        Self {
            acceleration_mode: VoicevoxAccelerationMode::from_rust(options.acceleration_mode),
            cpu_num_threads: options.cpu_num_threads,
            load_all_models: options.load_all_models,
        }
    };
}

impl From<VoicevoxInitializeOptions> for voicevox_core::InitializeOptions {
    fn from(value: VoicevoxInitializeOptions) -> Self {
        voicevox_core::InitializeOptions {
            acceleration_mode: value.acceleration_mode.into(),
            cpu_num_threads: value.cpu_num_threads,
            load_all_models: value.load_all_models,
        }
    }
}

impl ConstDefault for VoicevoxTtsOptions {
    const DEFAULT: Self = {
        let options = voicevox_core::TtsOptions::DEFAULT;
        Self {
            kana: options.kana,
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    };
}

impl From<VoicevoxTtsOptions> for voicevox_core::TtsOptions {
    fn from(options: VoicevoxTtsOptions) -> Self {
        Self {
            kana: options.kana,
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

impl ConstDefault for VoicevoxSynthesisOptions {
    const DEFAULT: Self = {
        let options = voicevox_core::TtsOptions::DEFAULT;
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    };
}

impl VoicevoxUserDictWord {
    pub(crate) unsafe fn try_into_word(&self) -> CApiResult<voicevox_core::UserDictWord> {
        Ok(UserDictWord::new(
            ensure_utf8(CStr::from_ptr(self.surface))?.to_string(),
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
