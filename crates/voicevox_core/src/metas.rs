use std::fmt::Display;

use super::*;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// スタイルIdの実体
pub type RawStyleId = u32;
/// スタイルId
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

pub type RawStyleVersion = String;

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

/// 音声合成モデルのメタ情報
pub type VoiceModelMeta = Vec<SpeakerMeta>;

/// スピーカーのメタ情報
#[derive(Deserialize, Serialize, Getters, Clone)]
pub struct SpeakerMeta {
    name: String,
    styles: Vec<StyleMeta>,
    version: StyleVersion,
    speaker_uuid: String,
}

/// スタイルのメタ情報
#[derive(Deserialize, Serialize, Getters, Clone)]
pub struct StyleMeta {
    id: StyleId,
    name: String,
}
