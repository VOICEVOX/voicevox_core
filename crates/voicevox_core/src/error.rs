use crate::engine::{FullContextLabelError, KanaParseError};

use super::*;
use result_code::VoicevoxResultCode::{self, *};
//use engine::
use thiserror::Error;

/*
 * 新しいエラーを定義したら、必ずresult_code.rsにあるVoicevoxResultCodeに対応するコードを定義し、
 * internal.rsにある変換関数に変換処理を加えること
 */

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

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_LOAD_MODEL_ERROR))]
    LoadModel(#[source] anyhow::Error),

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_LOAD_METAS_ERROR))]
    LoadMetas(#[source] anyhow::Error),

    #[error(
        "{},{0}",
        base_error_message(VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR)
    )]
    GetSupportedDevices(#[source] anyhow::Error),

    #[error("{}", base_error_message(VOICEVOX_RESULT_UNINITIALIZED_STATUS_ERROR))]
    UninitializedStatus,

    #[error(
        "{}: {speaker_id}",
        base_error_message(VOICEVOX_RESULT_INVALID_SPEAKER_ID_ERROR)
    )]
    InvalidSpeakerId { speaker_id: u32 },

    #[error(
        "{}: {model_index}",
        base_error_message(VOICEVOX_RESULT_INVALID_MODEL_INDEX_ERROR)
    )]
    InvalidModelIndex { model_index: usize },

    #[error("{}", base_error_message(VOICEVOX_RESULT_INFERENCE_ERROR))]
    InferenceFailed,

    #[error(
        "{},{0}",
        base_error_message(VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR)
    )]
    ExtractFullContextLabel(#[from] FullContextLabelError),

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_PARSE_KANA_ERROR))]
    ParseKana(#[from] KanaParseError),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NotLoadedOpenjtalkDict, Self::NotLoadedOpenjtalkDict)
            | (Self::GpuSupport, Self::GpuSupport)
            | (Self::UninitializedStatus, Self::UninitializedStatus)
            | (Self::InferenceFailed, Self::InferenceFailed) => true,
            (Self::LoadModel(e1), Self::LoadModel(e2))
            | (Self::LoadMetas(e1), Self::LoadMetas(e2))
            | (Self::GetSupportedDevices(e1), Self::GetSupportedDevices(e2)) => {
                e1.to_string() == e2.to_string()
            }
            (
                Self::InvalidSpeakerId {
                    speaker_id: speaker_id1,
                },
                Self::InvalidSpeakerId {
                    speaker_id: speaker_id2,
                },
            ) => speaker_id1 == speaker_id2,
            (
                Self::InvalidModelIndex {
                    model_index: model_index1,
                },
                Self::InvalidModelIndex {
                    model_index: model_index2,
                },
            ) => model_index1 == model_index2,
            (Self::ExtractFullContextLabel(e1), Self::ExtractFullContextLabel(e2)) => e1 == e2,
            (Self::ParseKana(e1), Self::ParseKana(e2)) => e1 == e2,
            _ => false,
        }
    }
}

fn base_error_message(result_code: VoicevoxResultCode) -> &'static str {
    let c_message: &'static str = crate::error_result_to_message(result_code);
    &c_message[..(c_message.len() - 1)]
}
