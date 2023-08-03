use self::engine::{FullContextLabelError, KanaParseError};
use self::result_code::VoicevoxResultCode::{self, *};
use super::*;
//use engine::
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

/// VOICEVOX COREのエラー。
#[derive(Error, Debug)]
pub enum Error {
    /*
     * エラーメッセージのベースとなる文字列は必ずbase_error_message関数を使用してVoicevoxResultCodeのエラー出力の内容と対応するようにすること
     */
    #[error(
        "{}",
        base_error_message(VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR)
    )]
    NotLoadedOpenjtalkDict,

    #[error("{}", base_error_message(VOICEVOX_RESULT_GPU_SUPPORT_ERROR))]
    GpuSupport,

    #[error("{} ({}): {source}", base_error_message(VOICEVOX_RESULT_LOAD_MODEL_ERROR), path.display())]
    LoadModel {
        path: PathBuf,
        #[source]
        source: anyhow::Error,
    },
    #[error("{} ({})", base_error_message(VOICEVOX_ALREADY_LOADED_MODEL_ERROR), path.display())]
    AlreadyLoadedModel { path: PathBuf },

    #[error("{} ({model_id:?})", base_error_message(VOICEVOX_UNLOADED_MODEL_ERROR))]
    UnloadedModel { model_id: VoiceModelId },

    #[error("{}({path}):{source}", base_error_message(VOICEVOX_OPEN_FILE_ERROR))]
    OpenFile {
        path: PathBuf,
        #[source]
        source: anyhow::Error,
    },

    #[error(
        "{}({path}):{source}",
        base_error_message(VOICEVOX_VVM_MODEL_READ_ERROR)
    )]
    VvmRead {
        path: PathBuf,
        #[source]
        source: anyhow::Error,
    },

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_LOAD_METAS_ERROR))]
    LoadMetas(#[source] anyhow::Error),

    #[error(
        "{},{0}",
        base_error_message(VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR)
    )]
    GetSupportedDevices(#[source] anyhow::Error),

    #[error(
        "{}: {style_id:?}",
        base_error_message(VOICEVOX_RESULT_INVALID_STYLE_ID_ERROR)
    )]
    InvalidStyleId { style_id: StyleId },

    #[error(
        "{}: {model_id:?}",
        base_error_message(VOICEVOX_RESULT_INVALID_MODEL_ID_ERROR)
    )]
    InvalidModelId { model_id: VoiceModelId },

    #[error("{}", base_error_message(VOICEVOX_RESULT_INFERENCE_ERROR))]
    InferenceFailed,

    #[error(
        "{},{0}",
        base_error_message(VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR)
    )]
    ExtractFullContextLabel(#[from] FullContextLabelError),

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_PARSE_KANA_ERROR))]
    ParseKana(#[from] KanaParseError),

    #[error("{}: {0}", base_error_message(VOICEVOX_LOAD_USER_DICT_ERROR))]
    LoadUserDict(String),

    #[error("{}: {0}", base_error_message(VOICEVOX_SAVE_USER_DICT_ERROR))]
    SaveUserDict(String),

    #[error("{}: {0}", base_error_message(VOICEVOX_UNKNOWN_USER_DICT_WORD_ERROR))]
    UnknownWord(Uuid),

    #[error("{}: {0}", base_error_message(VOICEVOX_USE_USER_DICT_ERROR))]
    UseUserDict(String),

    #[error("{}: {0}", base_error_message(VOICEVOX_INVALID_USER_DICT_WORD_ERROR))]
    InvalidWord(InvalidWordError),
}

fn base_error_message(result_code: VoicevoxResultCode) -> &'static str {
    let c_message: &'static str = crate::result_code::error_result_to_message(result_code);
    &c_message[..(c_message.len() - 1)]
}
