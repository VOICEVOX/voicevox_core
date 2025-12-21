use std::{
    convert::{self, Infallible},
    fmt,
    str::FromStr,
    sync::Arc,
};

use derive_more::AsRef;
use duplicate::duplicate_item;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize,
};
use typed_floats::{NonNaNFinite, PositiveFinite};
use typeshare::U53;

use crate::{error::InvalidQueryError, SamplingRate};

use super::super::Phoneme;

pub use self::optional_lyric::OptionalLyric;

/// 音符のID。
#[derive(
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    derive_more::Display,
    AsRef,
    Deserialize,
    Serialize,
)]
#[as_ref(str)]
pub struct NoteId(pub Arc<str>);

impl FromStr for NoteId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.into()))
    }
}

#[duplicate_item(
    T f;
    [ Arc<str> ] [ convert::identity ];
    [ &'_ str ] [ Into::into ];
    [ &'_ mut str ] [ Into::into ];
    [ String ] [ Into::into ];
)]
impl From<T> for NoteId {
    fn from(s: T) -> Self {
        Self(f(s))
    }
}

impl From<&'_ NoteId> for serde_json::Value {
    fn from(value: &'_ NoteId) -> Self {
        serde_json::to_value(value).expect("should be always serializable")
    }
}

impl FromStr for OptionalLyric {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).map_err(|()| {
            InvalidQueryError {
                what: "歌詞",
                value: Some(Box::new(s.to_owned())),
                source: None,
            }
            .into()
        })
    }
}

impl From<&'_ OptionalLyric> for serde_json::Value {
    fn from(value: &'_ OptionalLyric) -> Self {
        serde_json::to_value(value).expect("should be always serializable")
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
                OptionalLyric::new(s)
                    .map_err(|()| de::Error::invalid_value(Unexpected::Str(s), &self))
            }
        }
    }
}

/// 音符ごとの情報。
///
/// # Validation
///
/// この構造体の状態によっては、`Synthesizer`の各メソッドは[`ErrorKind::InvalidQuery`]を表わすエラーを返す。詳細は[`validate`メソッド]にて。
///
/// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
/// [`validate`メソッド]: Self::validate
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
#[non_exhaustive]
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

impl From<&'_ Note> for serde_json::Value {
    fn from(value: &'_ Note) -> Self {
        serde_json::to_value(value).expect("all of the fields should be always serializable")
    }
}

/// 楽譜情報。
///
/// # Validation
///
/// この構造体の状態によっては、`Synthesizer`の各メソッドは[`ErrorKind::InvalidQuery`]を表わすエラーを返す。詳細は[`validate`メソッド]にて。
///
/// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
/// [`validate`メソッド]: Self::validate
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Score {
    /// 音符のリスト。
    pub notes: Vec<Note>,
}

impl From<&'_ Score> for serde_json::Value {
    fn from(value: &'_ Score) -> Self {
        serde_json::to_value(value).expect("all of the fields should be always serializable")
    }
}

/// 音素の情報。
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct FramePhoneme {
    /// 音素。
    pub phoneme: Phoneme,

    /// 音素のフレーム長。
    pub frame_length: U53,

    /// 音符のID。
    pub note_id: Option<NoteId>,
}

impl From<&'_ FramePhoneme> for serde_json::Value {
    fn from(value: &'_ FramePhoneme) -> Self {
        serde_json::to_value(value).expect("all of the fields should be always serializable")
    }
}

/// フレームごとの音声合成用のクエリ。
///
/// # Serde
///
/// [Serde]においてはフィールド名はsnake\_caseの形ではなく、VOICEVOX
/// ENGINEに合わせる形でcamelCaseになっている。ただし今後の破壊的変更にて変わる可能性がある。[データのシリアライゼーション]を参照。
///
/// [Serde]: serde
/// [データのシリアライゼーション]: https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FrameAudioQuery {
    /// フレームごとの基本周波数。
    pub f0: Vec<PositiveFinite<f32>>,

    /// フレームごとの音量。
    pub volume: Vec<NonNaNFinite<f32>>,

    /// 音素のリスト。
    pub phonemes: Vec<FramePhoneme>,

    /// 全体の音量。
    ///
    /// # Serde
    ///
    /// [Serde]においては`volumeScale`という名前で扱われる。
    ///
    /// [Serde]: serde
    pub volume_scale: PositiveFinite<f32>,

    /// 音声データの出力サンプリングレート。
    ///
    /// # Serde
    ///
    /// [Serde]においては`outputSamplingRate`という名前で扱われる。
    ///
    /// [Serde]: serde
    pub output_sampling_rate: SamplingRate,

    /// 音声データをステレオ出力するか否か。
    ///
    /// # Serde
    ///
    /// [Serde]においては`outputStereo`という名前で扱われる。
    ///
    /// [Serde]: serde
    pub output_stereo: bool,
}

impl From<&'_ FrameAudioQuery> for serde_json::Value {
    fn from(value: &'_ FrameAudioQuery) -> Self {
        serde_json::to_value(value).expect("all of the fields should be always serializable")
    }
}

mod optional_lyric {
    use arrayvec::ArrayVec;
    use derive_more::AsRef;
    use serde_with::SerializeDisplay;
    use smol_str::SmolStr;

    use super::super::super::{
        acoustic_feature_extractor::{NonPauBaseVowel, OptionalConsonant},
        mora_mappings::MORA_KANA_TO_MORA_PHONEMES,
    };

    /// 音符の歌詞。`""`は[無音]。
    ///
    /// # Examples
    ///
    /// ```
    /// # use voicevox_core::OptionalLyric;
    /// #
    /// "ア".parse::<OptionalLyric>()?;
    /// "ヴォ".parse::<OptionalLyric>()?;
    /// "ん".parse::<OptionalLyric>()?; // 平仮名
    /// "".parse::<OptionalLyric>()?; // 無音
    ///
    /// "アア".parse::<OptionalLyric>().unwrap_err(); // 複数モーラは現状非対応
    /// # anyhow::Ok(())
    /// ```
    ///
    /// [無音]: crate::Phoneme::MorablePau
    #[derive(
        Clone,
        Default,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Debug,
        derive_more::Display,
        AsRef,
        SerializeDisplay,
    )]
    #[display("{text}")]
    pub struct OptionalLyric {
        /// # Invariant
        ///
        /// `phonemes` must come from this.
        #[as_ref(str)]
        text: SmolStr,

        /// # Invariant
        ///
        /// This must come from `text`.
        pub(super) phonemes: ArrayVec<(OptionalConsonant, NonPauBaseVowel), 1>,
    }

    impl OptionalLyric {
        /// [無音]。
        ///
        /// ```
        /// # use voicevox_core::OptionalLyric;
        /// #
        /// assert_eq!(OptionalLyric::default(), OptionalLyric::PAU);
        /// assert_eq!("", OptionalLyric::PAU.as_ref());
        /// ```
        ///
        /// [無音]: crate::Phoneme::MorablePau
        pub const PAU: Self = Self {
            text: SmolStr::new_static(""),
            phonemes: ArrayVec::new_const(),
        };

        pub(super) fn new(text: &str) -> Result<Self, ()> {
            if text.is_empty() {
                return Ok(Self::default());
            }

            let mora_kana = hira_to_kana(text).parse().map_err(|_| ())?;

            Ok(Self {
                text: text.into(),
                phonemes: [MORA_KANA_TO_MORA_PHONEMES[mora_kana]].into(),
            })
        }

        pub(in super::super) fn phonemes(
            &self,
        ) -> &ArrayVec<(OptionalConsonant, NonPauBaseVowel), 1> {
            &self.phonemes
        }
    }

    pub(super) fn hira_to_kana(s: &str) -> SmolStr {
        s.chars()
            .map(|c| match c {
                'ぁ'..='ゔ' => (u32::from(c) + 96).try_into().expect("should be OK"),
                c => c,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[rstest]
    #[case("ァ", "ァ")]
    #[case("ァ", "ぁ")]
    #[case("ヴ", "ゔ")]
    fn hira_to_kana_works(#[case] expected: &str, #[case] input: &str) {
        assert_eq!(expected, super::optional_lyric::hira_to_kana(input));
    }

    #[test]
    fn hira_to_kana_should_not_fail() {
        for c in 'ぁ'..='ゔ' {
            super::optional_lyric::hira_to_kana(&c.to_string());
        }
    }
}
