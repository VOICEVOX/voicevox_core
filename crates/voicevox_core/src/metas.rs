use std::fmt::Display;

use super::*;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// [`StyleId`]の実体。
///
/// [`StyleId`]: StyleId
pub type RawStyleId = u32;

/// スタイルID。
///
/// VOICEVOXにおける、ある[話者(speaker)]のある[スタイル(style)] (i.e. 声(voice))を指す。
///
/// [話者(speaker)]: SpeakerMeta
/// [スタイル(style)]: StyleMeta
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
}

/// **スタイル**(_style_)のメタ情報。
#[derive(Deserialize, Serialize, Getters, Clone)]
pub struct StyleMeta {
    /// スタイルID。
    id: StyleId,
    /// スタイル名。
    name: String,
}
