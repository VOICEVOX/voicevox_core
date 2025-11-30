mod validated;

use std::{fmt, num::NonZero, str::FromStr, sync::Arc};

use arrayvec::ArrayVec;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize,
};
use smol_str::SmolStr;
use typed_floats::{NonNaNFinite, PositiveFinite};
use typeshare::U53;

use super::super::{
    acoustic_feature_extractor::{MoraTail, OptionalConsonant},
    mora_list::MORA_KANA_TO_MORA_PHONEMES,
    Phoneme,
};

pub(crate) use self::validated::{KeyAndLyric, ValidatedNote};

/// 音符のID。
#[derive(Clone, Deserialize, Serialize)]
pub struct NoteId(pub Arc<str>);

#[derive(Clone)]
pub struct OptionalLyric(ArrayVec<(OptionalConsonant, MoraTail), 1>);

impl FromStr for OptionalLyric {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mora_kana = hira_to_kana(s).parse().map_err(Self::Err::from)?;
        return Ok(OptionalLyric(
            [MORA_KANA_TO_MORA_PHONEMES[mora_kana]].into(),
        ));

        fn hira_to_kana(s: &str) -> SmolStr {
            s.chars()
                .map(|c| match c {
                    'ぁ'..='ゔ' => (u32::from(c) + 96).try_into().expect("should be OK"),
                    c => c,
                })
                .collect()
        }
    }
}

impl<'de> Deserialize<'de> for OptionalLyric {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        return deserializer.deserialize_str(Visitor);

        struct Visitor;

        impl de::Visitor<'_> for Visitor {
            type Value = OptionalLyric;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(fmt, "a string that represents zero or one mora kana")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                s.parse()
                    .map_err(|_| de::Error::invalid_value(Unexpected::Str(s), &self))
            }
        }
    }
}

/// 音符ごとの情報。
#[derive(Clone)]
pub struct Note {
    /// ID。
    pub id: Option<NoteId>,

    /// 音階。
    pub key: Option<U53>,

    /// 歌詞。
    pub lyric: OptionalLyric,

    /// 音符のフレーム長。
    pub frame_length: U53,
}

/// 楽譜情報。
#[derive(Clone)]
pub struct Score {
    /// 音符のリスト。
    pub notes: Vec<Note>,
}

/// 音素の情報。
#[derive(Clone, Deserialize, Serialize)]
pub struct FramePhoneme {
    /// 音素。
    pub phoneme: Phoneme,

    /// 音素のフレーム長。
    pub frame_length: U53,

    /// 音符のID。
    pub note_id: Option<NoteId>,
}

/// フレームごとの音声合成用のクエリ。
///
/// # Serialization
///
/// VOICEVOX ENGINEと同じスキーマになっている。ただし今後の破壊的変更にて変わる可能性がある。[データのシリアライゼーション]を参照。
///
/// [データのシリアライゼーション]: https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameAudioQuery {
    /// フレームごとの基本周波数。
    pub f0: Vec<NonNaNFinite<f32>>,

    /// フレームごとの音量。
    pub volume: Vec<PositiveFinite<f32>>,

    /// 音素のリスト。
    pub phonemes: Vec<FramePhoneme>,

    /// 全体の音量。
    pub volume_scale: PositiveFinite<f32>,

    /// 音声データの出力サンプリングレート。
    pub output_sample_rate: NonZero<u32>,

    /// 音声データをステレオ出力するか否か。
    pub output_stereo: bool,
}
