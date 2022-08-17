use std::fmt::Display;

use crate::engine::{FullContextLabelError, KanaParseError};

use super::*;
use result_code::VoicevoxResultCode::{self, *};
//use engine::
use thiserror::Error;

/*
 * 新しいエラーを定義したら、必ずresult_code.rsにあるVoicevoxResultCodeに対応するコードを定義し、
 * internal.rsにある変換関数に変換処理を加えること
 */

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /*
     * エラーメッセージのベースとなる文字列は必ずbase_error_message関数を使用してVoicevoxResultCodeのエラー出力の内容と対応するようにすること
     */
    #[error("{}", base_error_message(VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT))]
    NotLoadedOpenjtalkDict,

    #[error("{}", base_error_message(VOICEVOX_RESULT_CANT_GPU_SUPPORT))]
    CantGpuSupport,

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_FAILED_LOAD_MODEL))]
    LoadModel(#[source] SourceError),

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_FAILED_LOAD_METAS))]
    LoadMetas(#[source] SourceError),

    #[error(
        "{},{0}",
        base_error_message(VOICEVOX_RESULT_FAILED_GET_SUPPORTED_DEVICES)
    )]
    GetSupportedDevices(#[source] SourceError),

    #[error("{}", base_error_message(VOICEVOX_RESULT_UNINITIALIZED_STATUS))]
    UninitializedStatus,

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_INVALID_SPEAKER_ID))]
    InvalidSpeakerId { speaker_id: usize },

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_INVALID_MODEL_INDEX))]
    InvalidModelIndex { model_index: usize },

    #[error("{}", base_error_message(VOICEVOX_RESULT_INFERENCE_FAILED))]
    InferenceFailed,

    #[error(
        "{},{0}",
        base_error_message(VOICEVOX_RESULT_FAILED_EXTRACT_FULL_CONTEXT_LABEL)
    )]
    FailedExtractFullContextLabel(#[from] FullContextLabelError),

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_FAILED_PARSE_KANA))]
    FailedParseKana(#[from] KanaParseError),
}

fn base_error_message(result_code: VoicevoxResultCode) -> &'static str {
    let c_message: &'static str = crate::voicevox_error_result_to_message(result_code);
    &c_message[..(c_message.len() - 1)]
}

#[derive(Debug)]
#[repr(transparent)]
pub struct SourceError(anyhow::Error);

impl SourceError {
    #[allow(dead_code)]
    pub fn new(source: anyhow::Error) -> Self {
        Self(source)
    }
}

impl Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> thiserror::private::AsDynError<'a> for SourceError {
    fn as_dyn_error(&self) -> &(dyn std::error::Error + 'a) {
        &*self.0
    }
}

impl PartialEq for SourceError {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_string() == other.0.to_string()
    }
}
impl AsRef<dyn std::error::Error + Send + Sync> for SourceError {
    fn as_ref(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        &*self.0
    }
}

impl<E: std::error::Error + Sync + Send + 'static> From<E> for SourceError {
    fn from(source: E) -> Self {
        Self(source.into())
    }
}
