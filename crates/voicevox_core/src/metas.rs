use std::fmt::Display;

use derive_getters::Getters;
use derive_new::new;
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};

pub fn merge<'a>(metas: impl IntoIterator<Item = &'a SpeakerMeta>) -> Vec<SpeakerMeta> {
    return metas
        .into_iter()
        .fold(IndexMap::<_, SpeakerMeta>::new(), |mut acc, speaker| {
            acc.entry(&speaker.speaker_uuid)
                .and_modify(|acc| acc.styles.extend(speaker.styles.clone()))
                .or_insert_with(|| speaker.clone());
            acc
        })
        .into_values()
        .update(|speaker| {
            speaker.styles.sort_by_key(|StyleMeta { id, .. }| {
                key(speaker
                    .style_order
                    .get_index_of(id)
                    .map(|i| i.try_into().unwrap()))
            });
        })
        .sorted_by_key(|&SpeakerMeta { speaker_order, .. }| key(speaker_order))
        .collect();

    fn key(order: Option<u32>) -> impl Ord {
        order
            .map(Into::into)
            .unwrap_or_else(|| u64::from(u32::MAX) + 1)
    }
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
#[derive(PartialEq, Eq, Clone, Copy, Ord, Hash, PartialOrd, Deserialize, Serialize, new, Debug)]
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
    /// 話者の順番。
    ///
    /// `SpeakerMeta`の列は、この値に対して昇順に並んでいるべきである。
    speaker_order: Option<u32>,
    /// 話者に属するスタイルの順番。
    ///
    /// [`styles`]はこの並びに沿うべきである。
    ///
    /// [`styles`]: Self::styles
    #[serde(default)]
    style_order: IndexSet<StyleId>,
}

impl SpeakerMeta {
    pub(crate) fn eq_except_styles(&self, other: &Self) -> bool {
        let Self {
            name: name1,
            styles: _,
            version: version1,
            speaker_uuid: speaker_uuid1,
            speaker_order: speaker_order1,
            style_order: style_order1,
        } = self;

        let Self {
            name: name2,
            styles: _,
            version: version2,
            speaker_uuid: speaker_uuid2,
            speaker_order: speaker_order2,
            style_order: style_order2,
        } = other;

        (name1, version1, speaker_uuid1, speaker_order1, style_order1)
            == (name2, version2, speaker_uuid2, speaker_order2, style_order2)
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
