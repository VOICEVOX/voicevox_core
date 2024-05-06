use crate::{
    engine::{FullContextLabelError, KanaParseError},
    user_dict::InvalidWordError,
    StyleId, StyleType, VoiceModelId,
};
//use engine::
use duplicate::duplicate_item;
use itertools::Itertools as _;
use std::{collections::BTreeSet, path::PathBuf};
use thiserror::Error;
use uuid::Uuid;

/// VOICEVOX COREのエラー。
#[derive(Error, Debug)]
#[error(transparent)]
pub struct Error(#[from] ErrorRepr);

#[duplicate_item(
    E;
    [ LoadModelError ];
    [ FullContextLabelError ];
    [ KanaParseError ];
    [ InvalidWordError ];
)]
impl From<E> for Error {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl Error {
    /// 対応する[`ErrorKind`]を返す。
    pub fn kind(&self) -> ErrorKind {
        match &self.0 {
            ErrorRepr::NotLoadedOpenjtalkDict => ErrorKind::NotLoadedOpenjtalkDict,
            ErrorRepr::GpuSupport => ErrorKind::GpuSupport,
            ErrorRepr::LoadModel(LoadModelError { context, .. }) => match context {
                LoadModelErrorKind::OpenZipFile => ErrorKind::OpenZipFile,
                LoadModelErrorKind::ReadZipEntry { .. } => ErrorKind::ReadZipEntry,
                LoadModelErrorKind::ModelAlreadyLoaded { .. } => ErrorKind::ModelAlreadyLoaded,
                LoadModelErrorKind::StyleAlreadyLoaded { .. } => ErrorKind::StyleAlreadyLoaded,
                LoadModelErrorKind::InvalidModelFormat { .. } => ErrorKind::InvalidModelFormat,
                LoadModelErrorKind::InvalidModelData => ErrorKind::InvalidModelData,
            },
            ErrorRepr::GetSupportedDevices(_) => ErrorKind::GetSupportedDevices,
            ErrorRepr::StyleNotFound { .. } => ErrorKind::StyleNotFound,
            ErrorRepr::ModelNotFound { .. } => ErrorKind::ModelNotFound,
            ErrorRepr::InferenceFailed { .. } => ErrorKind::InferenceFailed,
            ErrorRepr::ExtractFullContextLabel(_) => ErrorKind::ExtractFullContextLabel,
            ErrorRepr::ParseKana(_) => ErrorKind::ParseKana,
            ErrorRepr::LoadUserDict(_) => ErrorKind::LoadUserDict,
            ErrorRepr::SaveUserDict(_) => ErrorKind::SaveUserDict,
            ErrorRepr::WordNotFound(_) => ErrorKind::WordNotFound,
            ErrorRepr::UseUserDict(_) => ErrorKind::UseUserDict,
            ErrorRepr::InvalidWord(_) => ErrorKind::InvalidWord,
        }
    }
}

#[derive(Error, Debug)]
pub(crate) enum ErrorRepr {
    #[error("OpenJTalkの辞書が読み込まれていません")]
    NotLoadedOpenjtalkDict,

    #[error("GPU機能をサポートすることができません")]
    GpuSupport,

    #[error(transparent)]
    LoadModel(#[from] LoadModelError),

    #[error("サポートされているデバイス情報取得中にエラーが発生しました")]
    GetSupportedDevices(#[source] anyhow::Error),

    #[error(
        "`{style_id}` ([{style_types}])に対するスタイルが見つかりませんでした。音声モデルが\
         読み込まれていないか、読み込みが解除されています",
        style_types = style_types.iter().format(", ")
    )]
    StyleNotFound {
        style_id: StyleId,
        style_types: &'static BTreeSet<StyleType>,
    },

    #[error(
        "`{model_id}`に対する音声モデルが見つかりませんでした。読み込まれていないか、読み込みが既\
         に解除されています"
    )]
    ModelNotFound { model_id: VoiceModelId },

    #[error("推論に失敗しました")]
    InferenceFailed(#[source] anyhow::Error),

    #[error(transparent)]
    ExtractFullContextLabel(#[from] FullContextLabelError),

    #[error(transparent)]
    ParseKana(#[from] KanaParseError),

    #[error("ユーザー辞書を読み込めませんでした")]
    LoadUserDict(#[source] anyhow::Error),

    #[error("ユーザー辞書を書き込めませんでした")]
    SaveUserDict(#[source] anyhow::Error),

    #[error("ユーザー辞書に単語が見つかりませんでした: {0}")]
    WordNotFound(Uuid),

    #[error("OpenJTalkのユーザー辞書の設定に失敗しました")]
    UseUserDict(#[source] anyhow::Error),

    #[error(transparent)]
    InvalidWord(#[from] InvalidWordError),
}

/// エラーの種類。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ErrorKind {
    /// open_jtalk辞書ファイルが読み込まれていない。
    NotLoadedOpenjtalkDict,
    /// GPUモードがサポートされていない。
    GpuSupport,
    /// ZIPファイルを開くことに失敗した。
    OpenZipFile,
    /// ZIP内のファイルが読めなかった。
    ReadZipEntry,
    /// モデルの形式が不正。
    InvalidModelFormat,
    /// すでに読み込まれている音声モデルを読み込もうとした。
    ModelAlreadyLoaded,
    /// すでに読み込まれているスタイルを読み込もうとした。
    StyleAlreadyLoaded,
    /// 無効なモデルデータ。
    InvalidModelData,
    /// サポートされているデバイス情報取得に失敗した。
    GetSupportedDevices,
    /// スタイルIDに対するスタイルが見つからなかった。
    StyleNotFound,
    /// 音声モデルIDに対する音声モデルが見つからなかった。
    ModelNotFound,
    /// 推論に失敗した。
    InferenceFailed,
    /// コンテキストラベル出力に失敗した。
    ExtractFullContextLabel,
    /// AquesTalk風記法のテキストの解析に失敗した。
    ParseKana,
    /// ユーザー辞書を読み込めなかった。
    LoadUserDict,
    /// ユーザー辞書を書き込めなかった。
    SaveUserDict,
    /// ユーザー辞書に単語が見つからなかった。
    WordNotFound,
    /// OpenJTalkのユーザー辞書の設定に失敗した。
    UseUserDict,
    /// ユーザー辞書の単語のバリデーションに失敗した。
    InvalidWord,
}

pub(crate) type LoadModelResult<T> = std::result::Result<T, LoadModelError>;

/// 音声モデル読み込みのエラー。
#[derive(Error, Debug)]
#[error("`{path}`の読み込みに失敗しました: {context}")]
pub(crate) struct LoadModelError {
    pub(crate) path: PathBuf,
    pub(crate) context: LoadModelErrorKind,
    #[source]
    pub(crate) source: Option<anyhow::Error>,
}

#[derive(derive_more::Display, Debug)]
pub(crate) enum LoadModelErrorKind {
    #[display(fmt = "ZIPファイルとして開くことができませんでした")]
    OpenZipFile,
    #[display(fmt = "`{filename}`を読み取れませんでした")]
    ReadZipEntry { filename: String },
    #[display(fmt = "モデルの形式が不正です")]
    InvalidModelFormat,
    #[display(fmt = "モデル`{id}`は既に読み込まれています")]
    ModelAlreadyLoaded { id: VoiceModelId },
    #[display(fmt = "スタイル`{id}`は既に読み込まれています")]
    StyleAlreadyLoaded { id: StyleId },
    #[display(fmt = "モデルデータを読むことができませんでした")]
    InvalidModelData,
}
