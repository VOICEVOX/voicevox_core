use crate::{
    core::devices::DeviceAvailabilities,
    engine::{
        song::queries::Key,
        talk::{user_dict::InvalidWordError, KanaParseError},
        DEFAULT_SAMPLING_RATE,
    },
    StyleId, StyleType, VoiceModelId,
};
//use engine::
use duplicate::duplicate_item;
use itertools::Itertools as _;
use std::{collections::BTreeSet, fmt::Debug, path::PathBuf};
use thiserror::Error;
use uuid::Uuid;

/// VOICEVOX COREのエラー。
#[derive(Error, Debug)]
#[error(transparent)]
pub struct Error(#[from] pub(crate) ErrorRepr);

#[duplicate_item(
    E;
    [ LoadModelError ];
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
            ErrorRepr::GpuSupport(_) => ErrorKind::GpuSupport,
            ErrorRepr::InitInferenceRuntime { .. } => ErrorKind::InitInferenceRuntime,
            ErrorRepr::LoadModel(LoadModelError { context, .. }) => match context {
                LoadModelErrorKind::OpenZipFile => ErrorKind::OpenZipFile,
                LoadModelErrorKind::ReadZipEntry { .. } => ErrorKind::ReadZipEntry,
                LoadModelErrorKind::ModelAlreadyLoaded { .. } => ErrorKind::ModelAlreadyLoaded,
                LoadModelErrorKind::StyleAlreadyLoaded { .. } => ErrorKind::StyleAlreadyLoaded,
                LoadModelErrorKind::InvalidModelFormat => ErrorKind::InvalidModelFormat,
                LoadModelErrorKind::InvalidModelData => ErrorKind::InvalidModelData,
            },
            ErrorRepr::GetSupportedDevices(_) => ErrorKind::GetSupportedDevices,
            ErrorRepr::StyleNotFound { .. } => ErrorKind::StyleNotFound,
            ErrorRepr::ModelNotFound { .. } => ErrorKind::ModelNotFound,
            ErrorRepr::RunModel { .. } => ErrorKind::RunModel,
            ErrorRepr::AnalyzeText { .. } => ErrorKind::AnalyzeText,
            ErrorRepr::ParseKana(_) => ErrorKind::ParseKana,
            ErrorRepr::LoadUserDict(_) => ErrorKind::LoadUserDict,
            ErrorRepr::SaveUserDict(_) => ErrorKind::SaveUserDict,
            ErrorRepr::WordNotFound(_) => ErrorKind::WordNotFound,
            ErrorRepr::UseUserDict(_) => ErrorKind::UseUserDict,
            ErrorRepr::InvalidWord(_) => ErrorKind::InvalidWord,
            ErrorRepr::InvalidQuery { .. } => ErrorKind::InvalidQuery,
            ErrorRepr::IncompatibleQueries(_) => ErrorKind::IncompatibleQueries,
        }
    }
}

#[derive(Error, Debug)]
pub(crate) enum ErrorRepr {
    #[error("OpenJTalkの辞書が読み込まれていません")]
    NotLoadedOpenjtalkDict,

    #[error("GPU機能をサポートすることができません:\n{_0}")]
    GpuSupport(DeviceAvailabilities),

    #[error("{runtime_display_name}のロードまたは初期化ができませんでした")]
    InitInferenceRuntime {
        runtime_display_name: &'static str,
        #[source]
        source: anyhow::Error,
    },

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

    #[error(
        "正常に推論することができませんでした{}",
        note.as_ref().map(|s| format!("。NOTE: {s}")).unwrap_or_default()
    )]
    RunModel {
        note: Option<&'static str>,
        #[source]
        source: anyhow::Error,
    },

    #[error("入力テキストの解析に失敗しました")]
    AnalyzeText {
        text: String,
        #[source]
        source: anyhow::Error,
    },

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

    #[error(transparent)]
    InvalidQuery(#[from] InvalidQueryError),

    #[error(transparent)]
    IncompatibleQueries(IncompatibleQueriesError),
}

/// エラーの種類。
#[expect(
    clippy::manual_non_exhaustive,
    reason = "バインディングを作るときはexhaustiveとして扱いたい"
)]
#[cfg_attr(doc, doc(alias = "VoicevoxResultCode"))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ErrorKind {
    /// open_jtalk辞書ファイルが読み込まれていない。
    NotLoadedOpenjtalkDict,
    /// GPUモードがサポートされていない。
    GpuSupport,
    /// 推論ライブラリのロードまたは初期化ができなかった。
    InitInferenceRuntime,
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
    /// 推論に失敗した、もしくは推論結果が異常。
    RunModel,
    /// 入力テキストの解析に失敗した。
    AnalyzeText,
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
    /// [`AudioQuery`]、[`FrameAudioQuery`]、[`Score`]、もしくはその一部が不正。
    ///
    /// [`AudioQuery`]: crate::AudioQuery
    /// [`FrameAudioQuery`]: crate::FrameAudioQuery
    /// [`Score`]: crate::Score
    InvalidQuery,
    /// [`FrameAudioQuery`]と[`Score`]の組み合わせが不正。
    ///
    /// [`FrameAudioQuery`]: crate::FrameAudioQuery
    /// [`Score`]: crate::Score
    IncompatibleQueries,
    #[doc(hidden)]
    __NonExhaustive,
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
    #[display("ZIPファイルとして開くことができませんでした")]
    OpenZipFile,
    #[display("`{filename}`を読み取れませんでした")]
    ReadZipEntry { filename: String },
    #[display("モデルの形式が不正です")]
    InvalidModelFormat,
    #[display("モデル`{id}`は既に読み込まれています")]
    ModelAlreadyLoaded { id: VoiceModelId },
    #[display("スタイル`{id}`は既に読み込まれています")]
    StyleAlreadyLoaded { id: StyleId },
    #[display("モデルデータを読むことができませんでした")]
    InvalidModelData,
}

#[derive(Error, Debug)]
#[error(
    "不正な{what}です{value}",
    value = value
        .as_ref()
        .map(|value| format!(": {value:?}"))
        .unwrap_or_default()
)]
pub(crate) struct InvalidQueryError {
    pub(crate) what: &'static str,
    pub(crate) value: Option<Box<dyn Debug + Send + Sync + 'static>>,
    #[source]
    pub(crate) source: Option<InvalidQueryErrorSource>,
}

impl From<InvalidQueryError> for Error {
    fn from(err: InvalidQueryError) -> Self {
        ErrorRepr::InvalidQuery(err).into()
    }
}

#[derive(Error, Debug)]
pub(crate) enum InvalidQueryErrorSource {
    #[error("この二つの有無は一致していなければなりません")]
    PartiallyPresent,

    #[error("子音ではありません")]
    IsNotConsonant,

    #[error("子音です")]
    IsConsonant,

    #[error("\"sil\"を含む文字列である必要があります")]
    MustContainSil,

    #[error("`0`にすることはできません")]
    IsZero,

    #[error("0より大きい{DEFAULT_SAMPLING_RATE}の倍数でなければなりません")]
    IsNotMultipleOfBaseSamplingRate,

    #[error("lyricが空文字列の場合、keyはnullである必要があります。")]
    UnnecessaryKeyForPau,

    #[error("keyがnullの場合、lyricは空文字列である必要があります。")]
    MissingKeyForNonPau,

    #[error("{}以上{}以下である必要があります", Key::MIN, Key::MAX)]
    OutOfRangeKeyValue,

    #[error("{_0}")]
    NotInteger(serde_json::Error),

    #[error(r#"notesはpau (lyric="")から始まる必要があります"#)]
    InitialNoteMustBePau,

    #[error(transparent)]
    InvalidAsSuperset(Box<InvalidQueryError>),

    #[error("{fields}が不正です")]
    InvalidFields {
        fields: String,
        #[source]
        source: Box<InvalidQueryError>,
    },
}

#[derive(Clone, Copy, Error, Debug)]
#[error("不正な楽譜とFrameAudioQueryの組み合わせです。異なる音素ID列です")]
pub(crate) struct IncompatibleQueriesError;

impl From<IncompatibleQueriesError> for Error {
    fn from(err: IncompatibleQueriesError) -> Self {
        ErrorRepr::IncompatibleQueries(err).into()
    }
}
