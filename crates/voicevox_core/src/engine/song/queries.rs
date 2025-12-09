use std::{fmt, str::FromStr, sync::Arc};

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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NoteId(pub Arc<str>);

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
#[derive(Clone, Deserialize, Serialize)]
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
#[derive(Clone, Deserialize, Serialize)]
pub struct Score {
    /// 音符のリスト。
    pub notes: Vec<Note>,
}

/// 音素の情報。
#[derive(Clone, Debug, Deserialize, Serialize)]
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
/// # Serde
///
/// [Serde]においてはフィールド名はsnake\_caseの形ではなく、VOICEVOX
/// ENGINEに合わせる形でcamelCaseになっている。ただし今後の破壊的変更にて変わる可能性がある。[データのシリアライゼーション]を参照。
///
/// [Serde]: serde
/// [データのシリアライゼーション]: https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameAudioQuery {
    /// フレームごとの基本周波数。
    pub f0: Vec<NonNaNFinite<f32>>,

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

mod optional_lyric {
    use arrayvec::ArrayVec;
    use derive_more::AsRef;
    use serde_with::SerializeDisplay;
    use smol_str::SmolStr;

    use super::super::super::{
        acoustic_feature_extractor::{MoraTail, OptionalConsonant},
        mora_list::MORA_KANA_TO_MORA_PHONEMES,
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
    #[derive(Clone, Default, Debug, derive_more::Display, AsRef, SerializeDisplay)]
    #[display("{text}")]
    pub struct OptionalLyric {
        /// # Invariant
        ///
        /// `phonemes` must come from this.
        #[as_ref(str)]
        text: SmolStr,

        // TODO: `NonPauBaseVowel`型 (= a | i | u | e | o | cl | N) を導入する
        /// # Invariant
        ///
        /// This must come from `text`.
        pub(super) phonemes: ArrayVec<(OptionalConsonant, MoraTail), 1>,
    }

    impl OptionalLyric {
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

        pub(in super::super) fn phonemes(&self) -> &ArrayVec<(OptionalConsonant, MoraTail), 1> {
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
    #[test]
    fn hira_to_kana_should_not_fail() {
        for c in 'ぁ'..='ゔ' {
            super::optional_lyric::hira_to_kana(&c.to_string());
        }
    }
}
