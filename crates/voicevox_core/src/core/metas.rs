use std::fmt::{Debug, Display};

use derive_more::{AsMut, AsRef, Binary, From, Into, LowerHex, Octal, UpperHex};
use derive_new::new;
use indexmap::IndexMap;
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// [`speaker_uuid`]をキーとして複数の[`CharacterMeta`]をマージする。
///
/// マージする際キャラクターは[`CharacterMeta::order`]、スタイルは[`StyleMeta::order`]をもとに安定ソートされる。
/// `order`が無いキャラクターとスタイルは、そうでないものよりも後ろに置かれる。
///
/// [`speaker_uuid`]: CharacterMeta::speaker_uuid
pub fn merge<'a>(metas: impl IntoIterator<Item = &'a CharacterMeta>) -> Vec<CharacterMeta> {
    return metas
        .into_iter()
        .fold(IndexMap::<_, CharacterMeta>::new(), |mut acc, character| {
            acc.entry(&character.speaker_uuid)
                .and_modify(|acc| acc.styles.extend(character.styles.clone()))
                .or_insert_with(|| character.clone());
            acc
        })
        .into_values()
        .update(|character| {
            character
                .styles
                .sort_by_key(|&StyleMeta { order, .. }| key(order));
        })
        .sorted_by_key(|&CharacterMeta { order, .. }| key(order))
        .collect();

    fn key(order: Option<u32>) -> impl Ord {
        order
            .map(Into::into)
            .unwrap_or_else(|| u64::from(u32::MAX) + 1)
    }
}

/// スタイルID。
///
/// VOICEVOXにおける、ある[<i>キャラクター</i>]のある[<i>スタイル</i>]を指す。
///
/// [<i>キャラクター</i>]: CharacterMeta
/// [<i>スタイル</i>]: StyleMeta
#[derive(
    PartialEq,
    Eq,
    Clone,
    Copy,
    Ord,
    Hash,
    PartialOrd,
    From,
    Into,
    derive_more::FromStr,
    Deserialize,
    Serialize,
    new,
    Debug,
    UpperHex,
    LowerHex,
    Octal,
    Binary,
)]
#[cfg_attr(doc, doc(alias = "VoicevoxStyleId"))]
pub struct StyleId(pub u32);

impl Display for StyleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// [<i>キャラクター</i>]のバージョン。
///
/// [<i>キャラクター</i>]: CharacterMeta
#[derive(
    PartialEq, Eq, Clone, Ord, PartialOrd, Deserialize, Serialize, new, Hash, Debug, AsRef, AsMut,
)]
pub struct CharacterVersion(pub String);

impl Display for CharacterVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 音声モデルのメタ情報。
pub type VoiceModelMeta = Vec<CharacterMeta>;

/// <i>キャラクター</i>のメタ情報。
#[derive(Deserialize, Serialize, Clone, Debug)]
#[non_exhaustive]
pub struct CharacterMeta {
    /// キャラクター名。
    pub name: String,
    /// キャラクターに属するスタイル。
    pub styles: Vec<StyleMeta>,
    /// キャラクターのバージョン。
    pub version: CharacterVersion,
    /// キャラクターのUUID。
    pub speaker_uuid: String,
    /// キャラクターの順番。
    ///
    /// `CharacterMeta`の列は、この値に対して昇順に並んでいるべきである。
    pub order: Option<u32>,
}

impl CharacterMeta {
    /// # Panics
    ///
    /// `speaker_uuid`が異なるときパニックする。
    pub(super) fn warn_diff_except_styles(&self, other: &Self) {
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

/// <i>スタイル</i>のメタ情報。
#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
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
    /// [`CharacterMeta::styles`]は、この値に対して昇順に並んでいるべきである。
    pub order: Option<u32>,
}

/// [<i>スタイル</i>]に対応するモデルの種類。
///
/// # Serde
///
/// [Serde]においては各バリアント名はsnake\_caseとなる。
///
/// [<i>スタイル</i>]: StyleMeta
/// [Serde]: serde
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
    ///
    /// # Serde
    ///
    /// [Serde]においては`"talk"`という値で表される。
    ///
    /// [Serde]: serde
    #[default]
    Talk,

    /// 歌唱音声合成用のクエリの作成が可能。
    ///
    /// # Serde
    ///
    /// [Serde]においては`"singing_teacher"`という値で表される。
    ///
    /// [Serde]: serde
    SingingTeacher,

    /// 歌唱音声合成が可能。
    ///
    /// # Serde
    ///
    /// [Serde]においては`"frame_decode"`という値で表される。
    ///
    /// [Serde]: serde
    FrameDecode,

    /// 歌唱音声合成用のクエリの作成と歌唱音声合成が可能。
    ///
    /// # Serde
    ///
    /// [Serde]においては`"sing"`という値で表される。
    ///
    /// [Serde]: serde
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
