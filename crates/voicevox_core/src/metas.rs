use std::fmt::Display;

use derive_getters::Getters;
use derive_new::new;
use serde::{Deserialize, Serialize};

/// [`StyleId`]の実体。
///
/// [`StyleId`]: StyleId
pub type RawStyleId = u32;

/// スタイルID。
///
/// VOICEVOXにおける、ある[**話者**(_speaker_)]のある[**スタイル**(_style_)]を指す。
///
/// [**話者**(_speaker_)]: SpeakerMeta
/// [**スタイル**(_style_)]: StyleMeta
#[derive(PartialEq, Eq, Clone, Copy, Ord, PartialOrd, Deserialize, Serialize, new, Debug)]
pub struct StyleId(RawStyleId);

impl StyleId {
    pub fn raw_id(self) -> RawStyleId {
        self.0
    }
}

impl Display for StyleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_id())
    }
}

/// [`StyleVersion`]の実体。
///
/// [`StyleVersion`]: StyleVersion
pub type RawStyleVersion = String;

/// スタイルのバージョン。
#[derive(PartialEq, Eq, Clone, Ord, PartialOrd, Deserialize, Serialize, new, Debug)]
pub struct StyleVersion(RawStyleVersion);

impl StyleVersion {
    pub fn raw_version(&self) -> &RawStyleVersion {
        &self.0
    }
}

impl Display for StyleVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_version())
    }
}

/// 音声モデルのメタ情報。
pub type VoiceModelMeta = Vec<SpeakerMeta>;

/// **話者**(_speaker_)のメタ情報。
#[derive(Deserialize, Serialize, Getters, Clone)]
pub struct SpeakerMeta {
    /// 話者名。
    name: String,
    /// 話者に属するスタイル。
    styles: Vec<StyleMeta>,
    /// 話者のバージョン。
    version: StyleVersion,
    /// 話者のUUID。
    speaker_uuid: String,
    /// 話者の対応機能。
    #[serde(default)]
    supported_features: SpeakerSupportedFeatures,
}

/// **スタイル**(_style_)のメタ情報。
#[derive(Deserialize, Serialize, Getters, Clone)]
pub struct StyleMeta {
    /// スタイルID。
    id: StyleId,
    /// スタイル名。
    name: String,
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct SpeakerSupportedFeatures {
    pub(crate) permitted_synthesis_morphing: PermittedSynthesisMorphing,
}

#[derive(Deserialize, Serialize, Default, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum PermittedSynthesisMorphing {
    /// 全て許可。
    All,

    /// 同じ話者内でのみ許可。
    SelfOnly,

    /// 全て禁止。
    #[default]
    Nothing,
}
