use std::fmt::{Debug, Display};

use derive_new::new;
use indexmap::IndexMap;
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// [`speaker_uuid`]をキーとして複数の[`SpeakerMeta`]をマージする。
///
/// マージする際話者は[`SpeakerMeta::order`]、スタイルは[`StyleMeta::order`]をもとに安定ソートされる。
/// `order`が無い話者とスタイルは、そうでないものよりも後ろに置かれる。
///
/// [`speaker_uuid`]: SpeakerMeta::speaker_uuid
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
            speaker
                .styles
                .sort_by_key(|&StyleMeta { order, .. }| key(order));
        })
        .sorted_by_key(|&SpeakerMeta { order, .. }| key(order))
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
#[derive(
    PartialEq,
    Eq,
    Clone,
    Copy,
    Ord,
    Hash,
    PartialOrd,
    derive_more::FromStr,
    Deserialize,
    Serialize,
    new,
    Debug,
)]
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

/// [`SpeakerVersion`]の実体。
pub type RawSpeakerVersion = String;

/// [**話者**(_speaker_)]のバージョン。
///
/// [**話者**(_speaker_)]: SpeakerMeta
#[derive(PartialEq, Eq, Clone, Ord, PartialOrd, Deserialize, Serialize, new, Debug)]
pub struct SpeakerVersion(RawSpeakerVersion);

impl SpeakerVersion {
    pub fn raw_version(&self) -> &RawSpeakerVersion {
        &self.0
    }
}

impl Display for SpeakerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_version())
    }
}

/// 音声モデルのメタ情報。
pub type VoiceModelMeta = Vec<SpeakerMeta>;

/// **話者**(_speaker_)のメタ情報。
#[derive(Deserialize, Serialize, Clone)]
#[non_exhaustive]
pub struct SpeakerMeta {
    /// 話者名。
    pub name: String,
    /// 話者に属するスタイル。
    pub styles: Vec<StyleMeta>,
    /// 話者のバージョン。
    pub version: SpeakerVersion,
    /// 話者のUUID。
    pub speaker_uuid: String,
    /// 話者の順番。
    ///
    /// `SpeakerMeta`の列は、この値に対して昇順に並んでいるべきである。
    pub order: Option<u32>,
}

impl SpeakerMeta {
    /// # Panics
    ///
    /// `speaker_uuid`が異なるときパニックする。
    pub(crate) fn warn_diff_except_styles(&self, other: &Self) {
        let Self {
            name: name1,
            styles: _,
            version: version1,
            speaker_uuid: speaker_uuid1,
            order: order1,
        } = self;

        let Self {
            name: name2,
            styles: _,
            version: version2,
            speaker_uuid: speaker_uuid2,
            order: order2,
        } = other;

        if speaker_uuid1 != speaker_uuid2 {
            panic!("must be equal: {speaker_uuid1} != {speaker_uuid2:?}");
        }

        warn_diff(speaker_uuid1, "name", name1, name2);
        warn_diff(speaker_uuid1, "version", version1, version2);
        warn_diff(speaker_uuid1, "order", order1, order2);

        fn warn_diff<T: PartialEq + Debug>(
            speaker_uuid: &str,
            field_name: &str,
            left: &T,
            right: &T,
        ) {
            if left != right {
                warn!("`{speaker_uuid}`: different `{field_name}` ({left:?} != {right:?})");
            }
        }
    }
}

/// **スタイル**(_style_)のメタ情報。
#[derive(Deserialize, Serialize, Clone)]
#[non_exhaustive]
pub struct StyleMeta {
    /// スタイルID。
    pub id: StyleId,
    /// スタイル名。
    pub name: String,
    /// スタイルに対応するモデルの種類。
    #[serde(default)]
    pub r#type: StyleType,
    /// スタイルの順番。
    ///
    /// [`SpeakerMeta::styles`]は、この値に対して昇順に並んでいるべきである。
    pub order: Option<u32>,
}

/// **スタイル**(_style_)に対応するモデルの種類。
#[derive(
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    strum::Display,
    Deserialize,
    Serialize,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum StyleType {
    /// 音声合成クエリの作成と音声合成が可能。
    #[default]
    Talk,

    /// 歌唱音声合成用のクエリの作成が可能。
    SingingTeacher,

    /// 歌唱音声合成が可能。
    FrameDecode,

    /// 歌唱音声合成用のクエリの作成と歌唱音声合成が可能。
    Sing,
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use serde_json::json;

    #[test]
    fn merge_works() -> anyhow::Result<()> {
        static INPUT: LazyLock<serde_json::Value> = LazyLock::new(|| {
            json!([
                {
                    "name": "B",
                    "styles": [
                        {
                            "id": 3,
                            "name": "B_1",
                            "type": "talk",
                            "order": 0
                        }
                    ],
                    "version": "0.0.0",
                    "speaker_uuid": "f34ab151-c0f5-4e0a-9ad2-51ce30dba24d",
                    "order": 1
                },
                {
                    "name": "A",
                    "styles": [
                        {
                            "id": 2,
                            "name": "A_3",
                            "type": "talk",
                            "order": 2
                        }
                    ],
                    "version": "0.0.0",
                    "speaker_uuid": "d6fd707c-a451-48e9-8f00-fe9ee3bf6264",
                    "order": 0
                },
                {
                    "name": "A",
                    "styles": [
                        {
                            "id": 1,
                            "name": "A_1",
                            "type": "talk",
                            "order": 0
                        },
                        {
                            "id": 0,
                            "name": "A_2",
                            "type": "talk",
                            "order": 1
                        }
                    ],
                    "version": "0.0.0",
                    "speaker_uuid": "d6fd707c-a451-48e9-8f00-fe9ee3bf6264",
                    "order": 0
                }
            ])
        });

        static EXPECTED: LazyLock<serde_json::Value> = LazyLock::new(|| {
            json!([
                {
                    "name": "A",
                    "styles": [
                        {
                            "id": 1,
                            "name": "A_1",
                            "type": "talk",
                            "order": 0
                        },
                        {
                            "id": 0,
                            "name": "A_2",
                            "type": "talk",
                            "order": 1
                        },
                        {
                            "id": 2,
                            "name": "A_3",
                            "type": "talk",
                            "order": 2
                        }
                    ],
                    "version": "0.0.0",
                    "speaker_uuid": "d6fd707c-a451-48e9-8f00-fe9ee3bf6264",
                    "order": 0
                },
                {
                    "name": "B",
                    "styles": [
                        {
                            "id": 3,
                            "name": "B_1",
                            "type": "talk",
                            "order": 0
                        }
                    ],
                    "version": "0.0.0",
                    "speaker_uuid": "f34ab151-c0f5-4e0a-9ad2-51ce30dba24d",
                    "order": 1
                }
            ])
        });

        let input = &serde_json::from_value::<Vec<_>>(INPUT.clone())?;
        let actual = serde_json::to_value(super::merge(input))?;

        pretty_assertions::assert_eq!(*EXPECTED, actual);
        Ok(())
    }
}
