use self::engine::{FullContextLabelError, KanaParseError};
use super::*;
//use engine::
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

/// VOICEVOX COREのエラー。
#[derive(Error, Debug)]
pub enum Error {
    #[error("OpenJTalkの辞書が読み込まれていません")]
    NotLoadedOpenjtalkDict,

    #[error("GPU機能をサポートすることができません")]
    GpuSupport,

    #[error(transparent)]
    LoadModel(#[from] LoadModelError),

    #[error("Modelが読み込まれていません ({model_id:?})")]
    UnloadedModel { model_id: VoiceModelId },

    #[error("サポートされているデバイス情報取得中にエラーが発生しました,{0}")]
    GetSupportedDevices(#[source] anyhow::Error),

    #[error("無効なspeaker_idです: {style_id:?}")]
    InvalidStyleId { style_id: StyleId },

    #[error("無効なmodel_idです: {model_id:?}")]
    InvalidModelId { model_id: VoiceModelId },

    #[error("推論に失敗しました")]
    InferenceFailed,

    #[error("入力テキストからのフルコンテキストラベル抽出に失敗しました,{0}")]
    ExtractFullContextLabel(#[from] FullContextLabelError),

    #[error("入力テキストをAquesTalk風記法としてパースすることに失敗しました,{0}")]
    ParseKana(#[from] KanaParseError),

    #[error("ユーザー辞書を読み込めませんでした: {0}")]
    LoadUserDict(String),

    #[error("ユーザー辞書を書き込めませんでした: {0}")]
    SaveUserDict(String),

    #[error("ユーザー辞書に単語が見つかりませんでした: {0}")]
    UnknownWord(Uuid),

    #[error("OpenJTalkのユーザー辞書の設定に失敗しました: {0}")]
    UseUserDict(String),

    #[error("ユーザー辞書の単語のバリデーションに失敗しました: {0}")]
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
    #[display(fmt = "ZIPファイルとして開くことができませんでした")]
    OpenZipFile,
    #[display(fmt = "`{filename}`を読み取れませんでした")]
    ReadZipEntry { filename: String },
    #[display(fmt = "モデル`{id}`は既に読み込まれています")]
    ModelAlreadyLoaded { id: VoiceModelId },
    #[display(fmt = "スタイル`{id}`は既に読み込まれています")]
    StyleAlreadyLoaded { id: StyleId },
    #[display(fmt = "モデルデータを読むことができませんでした")]
    InvalidModelData,
}
