use std::fmt::Display;

use derive_getters::Getters;
use derive_new::new;
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};

pub fn merge<'a>(metas: impl IntoIterator<Item = &'a SpeakerMeta>) -> Vec<SpeakerMeta> {
    metas
        .into_iter()
        .into_grouping_map_by(|speaker| &speaker.speaker_uuid)
        .aggregate::<_, SpeakerMeta>(|acc, _, speaker| {
            Some(
                acc.map(|mut acc| {
                    acc.styles.extend(speaker.styles.clone());
                    acc
                })
                .unwrap_or_else(|| speaker.clone()),
            )
        })
        .into_values()
        .sorted_by_key(|SpeakerMeta { styles, .. }| {
            styles.iter().map(|&StyleMeta { id, .. }| id).min()
        })
        .collect()
}

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
}

impl SpeakerMeta {
    pub(crate) fn eq_except_styles(&self, other: &Self) -> bool {
        let Self {
            name: name1,
            styles: _,
            version: version1,
            speaker_uuid: speaker_uuid1,
        } = self;

        let Self {
            name: name2,
            styles: _,
            version: version2,
            speaker_uuid: speaker_uuid2,
        } = other;

        (name1, version1, speaker_uuid1) == (name2, version2, speaker_uuid2)
    }
}

/// **スタイル**(_style_)のメタ情報。
#[derive(Deserialize, Serialize, Getters, Clone)]
pub struct StyleMeta {
    /// スタイルID。
    id: StyleId,
    /// スタイル名。
    name: String,
}
