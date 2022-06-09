use super::*;
use c_export::VoicevoxResultCode::{self, *};
use thiserror::Error;

/*
 * 新しいエラーを定義したら、必ずc_export.rsにあるVoicevoxResultCodeに対応するコードを定義し、
 * internal.rsにある変換関数に変換処理を加えること
 */

#[derive(Error, Debug)]
pub enum Error {
    /*
     * エラーメッセージのベースとなる文字列は必ずbase_error_message関数を使用してVoicevoxResultCodeのエラー出力の内容と対応するようにすること
     */
    #[error("{}", base_error_message(VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT))]
    // TODO:仮実装がlinterエラーにならないようにするための属性なのでこのenumが正式に使われる際にallow(dead_code)を取り除くこと
    #[allow(dead_code)]
    NotLoadedOpenjtalkDict,

    #[error("{}", base_error_message(VOICEVOX_RESULT_CANT_GPU_SUPPORT))]
    CantGpuSupport,

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_FAILED_LOAD_MODEL))]
    LoadModel(#[source] anyhow::Error),

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_FAILED_LOAD_METAS))]
    LoadMetas(#[source] anyhow::Error),

    #[error(
        "{},{0}",
        base_error_message(VOICEVOX_RESULT_FAILED_GET_SUPPORTED_DEVICES)
    )]
    GetSupportedDevices(#[source] anyhow::Error),

    #[error("{}", base_error_message(VOICEVOX_RESULT_UNINITIALIZED_STATUS))]
    UninitializedStatus,

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_INVALID_SPEAKER_ID))]
    InvalidSpeakerId(usize),

    #[error("{},{0}", base_error_message(VOICEVOX_RESULT_INVALID_MODEL_INDEX))]
    InvalidModelIndex(usize),
}

fn base_error_message(result_code: VoicevoxResultCode) -> &'static str {
    let c_message: &'static str = internal::voicevox_error_result_to_message(result_code);
    &c_message[..(c_message.len() - 1)]
}
