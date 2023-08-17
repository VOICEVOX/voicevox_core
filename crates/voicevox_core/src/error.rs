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

    #[error(transparent)]
    LoadModel(#[from] LoadModelError),

    #[error(
        "{} ({model_id:?})",
        base_error_message(VOICEVOX_RESULT_UNLOADED_MODEL_ERROR)
    )]
    UnloadedModel { model_id: VoiceModelId },

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

    #[error("{}: {0}", base_error_message(VOICEVOX_RESULT_LOAD_USER_DICT_ERROR))]
    LoadUserDict(String),

    #[error("{}: {0}", base_error_message(VOICEVOX_RESULT_SAVE_USER_DICT_ERROR))]
    SaveUserDict(String),

    #[error(
        "{}: {0}",
        base_error_message(VOICEVOX_RESULT_UNKNOWN_USER_DICT_WORD_ERROR)
    )]
    UnknownWord(Uuid),

    #[error("{}: {0}", base_error_message(VOICEVOX_RESULT_USE_USER_DICT_ERROR))]
    UseUserDict(String),

    #[error(
        "{}: {0}",
        base_error_message(VOICEVOX_RESULT_INVALID_USER_DICT_WORD_ERROR)
    )]
    InvalidWord(InvalidWordError),
}

pub(crate) type LoadModelResult<T> = std::result::Result<T, LoadModelError>;

/// 音声モデル読み込みのエラー。
#[derive(Error, Debug)]
#[error(
    "`{path}`の読み込みに失敗しました: {context}{}",
    source.as_ref().map(|e| format!(": {e}")).unwrap_or_default())
]
pub struct LoadModelError {
    pub(crate) path: PathBuf,
    pub(crate) context: LoadModelErrorKind,
    #[source]
    pub(crate) source: Option<anyhow::Error>,
}

impl LoadModelError {
    pub fn context(&self) -> &LoadModelErrorKind {
        &self.context
    }
}

#[derive(derive_more::Display, Debug)]
pub enum LoadModelErrorKind {
    //#[display(fmt = "{}", "base_error_message(VOICEVOX_RESULT_OPEN_ZIP_FILE_ERROR)")]
    #[display(fmt = "ZIPファイルとして開くことができませんでした")]
    OpenZipFile,
    //#[display(fmt = "{}", "base_error_message(VOICEVOX_RESULT_READ_ZIP_ENTRY_ERROR)")]
    #[display(fmt = "`{filename}`を読み取れませんでした")]
    ReadZipEntry { filename: String },
    //#[display(fmt = "{}", "base_error_message(VOICEVOX_RESULT_MODEL_ALREADY_LOADED_ERROR)")]
    #[display(fmt = "モデル`{id}`は既に読み込まれています")]
    ModelAlreadyLoaded { id: VoiceModelId },
    //#[display(fmt = "{}", "base_error_message(VOICEVOX_RESULT_STYLE_ALREADY_LOADED_ERROR)")]
    #[display(fmt = "スタイル`{id}`は既に読み込まれています")]
    StyleAlreadyLoaded { id: StyleId },
    #[display(
        fmt = "{}",
        "base_error_message(VOICEVOX_RESULT_INVALID_MODEL_DATA_ERROR)"
    )]
    InvalidModelData,
}

fn base_error_message(result_code: VoicevoxResultCode) -> &'static str {
    let c_message: &'static str = crate::result_code::error_result_to_message(result_code);
    &c_message[..(c_message.len() - 1)]
}
