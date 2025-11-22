use std::{borrow::Cow, num::NonZero};

use serde::{Deserialize, Serialize};

use crate::error::{ErrorRepr, InvalidQueryErrorKind};

use super::super::acoustic_feature_extractor::Phoneme;

pub(crate) use self::sampling_rate::SamplingRate;

/* 各フィールドのjsonフィールド名はsnake_caseとする*/

/// モーラ（子音＋母音）ごとの情報。
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct Mora {
    /// 文字。
    pub text: String,
    /// 子音の音素。
    pub consonant: Option<String>,
    /// 子音の音長。
    pub consonant_length: Option<f32>,
    /// 母音の音素。
    pub vowel: String,
    /// 母音の音長。
    pub vowel_length: f32,
    /// 音高。
    pub pitch: f32,
}

impl Mora {
    pub fn validate(&self) -> crate::Result<()> {
        self.to_validated().map(|_| ())
    }

    fn to_validated(&self) -> crate::Result<ValidatedMora<'_>> {
        ValidatedMora::new(self).map_err(|kind| {
            ErrorRepr::InvalidQuery {
                what: "モーラ",
                kind,
            }
            .into()
        })
    }
}

/// AccentPhrase (アクセント句ごとの情報)。
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct AccentPhrase {
    /// モーラの配列。
    pub moras: Vec<Mora>,
    /// アクセント箇所。
    pub accent: usize,
    /// 後ろに無音を付けるかどうか。
    pub pause_mora: Option<Mora>,
    /// 疑問系かどうか。
    #[serde(default)]
    pub is_interrogative: bool,
}

impl AccentPhrase {
    pub(super) fn set_pause_mora(&mut self, pause_mora: Option<Mora>) {
        self.pause_mora = pause_mora;
    }

    pub(super) fn set_is_interrogative(&mut self, is_interrogative: bool) {
        self.is_interrogative = is_interrogative;
    }
}

impl AccentPhrase {
    pub fn validate(&self) -> crate::Result<()> {
        self.to_validated().map(|_| ())
    }

    pub(crate) fn to_validated(&self) -> crate::Result<ValidatedAccentPhrase<'_>> {
        ValidatedAccentPhrase::new(self).map_err(|kind| {
            ErrorRepr::InvalidQuery {
                what: "アクセント句",
                kind,
            }
            .into()
        })
    }
}

/// AudioQuery (音声合成用のクエリ)。
///
/// # Serialization
///
/// VOICEVOX ENGINEと同じスキーマになっている。ただし今後の破壊的変更にて変わる可能性がある。[データのシリアライゼーション]を参照。
///
/// [データのシリアライゼーション]: https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AudioQuery {
    /// アクセント句の配列。
    pub accent_phrases: Vec<AccentPhrase>,
    /// 全体の話速。
    #[serde(rename = "speedScale")]
    pub speed_scale: f32,
    /// 全体の音高。
    #[serde(rename = "pitchScale")]
    pub pitch_scale: f32,
    /// 全体の抑揚。
    #[serde(rename = "intonationScale")]
    pub intonation_scale: f32,
    /// 全体の音量。
    #[serde(rename = "volumeScale")]
    pub volume_scale: f32,
    /// 音声の前の無音時間。
    #[serde(rename = "prePhonemeLength")]
    pub pre_phoneme_length: f32,
    /// 音声の後の無音時間。
    #[serde(rename = "postPhonemeLength")]
    pub post_phoneme_length: f32,
    /// 音声データの出力サンプリングレート。
    #[serde(rename = "outputSamplingRate")]
    pub output_sampling_rate: u32,
    /// 音声データをステレオ出力するか否か。
    #[serde(rename = "outputStereo")]
    pub output_stereo: bool,
    /// \[読み取り専用\] AquesTalk風記法。
    ///
    /// [`Synthesizer::create_audio_query`]が返すもののみ`Some`となる。入力としてのAudioQueryでは無視され
    /// る。
    ///
    /// [`Synthesizer::create_audio_query`]: crate::blocking::Synthesizer::create_audio_query
    pub kana: Option<String>,
}

impl AudioQuery {
    pub fn validate(&self) -> crate::Result<()> {
        self.to_validated().map(|_| ())
    }

    pub(crate) fn to_validated(&self) -> crate::Result<ValidatedAudioQuery<'_>> {
        ValidatedAudioQuery::new(self).map_err(|kind| {
            ErrorRepr::InvalidQuery {
                what: "AudioQuery",
                kind,
            }
            .into()
        })
    }

    pub(crate) fn with_kana(self, kana: Option<String>) -> Self {
        Self { kana, ..self }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct ValidatedMora<'original> {
    pub(crate) text: Cow<'original, str>,
    pub(crate) consonant: Option<LengthedPhoneme<'original>>,
    pub(crate) vowel: LengthedPhoneme<'original>,
    pub(crate) pitch: f32,
}

impl<'original> ValidatedMora<'original> {
    fn new(original: &'original Mora) -> Result<Self, InvalidQueryErrorKind> {
        let consonant = match (&original.consonant, original.consonant_length) {
            (Some(phoneme), Some(length)) => Some(LengthedPhoneme::new(phoneme, length)?),
            (None, None) => None,
            (Some(_), None) => return Err(InvalidQueryErrorKind::MissingConsonantLength),
            (None, Some(_)) => return Err(InvalidQueryErrorKind::MissingConsonantPhoneme),
        };

        let vowel = LengthedPhoneme::new(&original.vowel, original.vowel_length)?;

        let text = (&original.text).into();

        let Mora { pitch, .. } = *original;

        Ok(Self {
            text,
            consonant,
            vowel,
            pitch,
        })
    }

    fn into_owned(self) -> ValidatedMora<'static> {
        let Self {
            text,
            consonant,
            vowel,
            pitch,
        } = self;
        let text = text.into_owned().into();
        let consonant = consonant.map(LengthedPhoneme::into_owned);
        let vowel = vowel.into_owned();
        ValidatedMora {
            text,
            consonant,
            vowel,
            pitch,
        }
    }
}

impl From<ValidatedMora<'_>> for Mora {
    fn from(
        ValidatedMora {
            text,
            consonant,
            vowel,
            pitch,
        }: ValidatedMora<'_>,
    ) -> Self {
        Self {
            text: text.into_owned(),
            consonant: consonant
                .as_ref()
                .map(|LengthedPhoneme { phoneme, .. }| phoneme.to_string()),
            consonant_length: consonant.map(|LengthedPhoneme { length, .. }| length),
            vowel: vowel.phoneme.to_string(),
            vowel_length: vowel.length,
            pitch,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct LengthedPhoneme<'original> {
    pub(crate) phoneme: Cow<'original, Phoneme>,
    pub(crate) length: f32,
}

impl LengthedPhoneme<'static> {
    fn new(phoneme: &str, length: f32) -> Result<Self, InvalidQueryErrorKind> {
        let phoneme = Cow::Owned(
            phoneme
                .parse()
                .map_err(|_| InvalidQueryErrorKind::InvalidPhoneme(phoneme.to_owned()))?,
        );
        Ok(Self { phoneme, length })
    }
}

impl LengthedPhoneme<'_> {
    fn into_owned(self) -> LengthedPhoneme<'static> {
        let Self { phoneme, length } = self;
        let phoneme = Cow::Owned(phoneme.into_owned());
        LengthedPhoneme { phoneme, length }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct ValidatedAccentPhrase<'original> {
    pub(crate) moras: Vec<ValidatedMora<'original>>,
    pub(crate) accent: NonZero<usize>,
    pub(crate) pause_mora: Option<ValidatedMora<'original>>,
    pub(crate) is_interrogative: bool,
}

impl<'original> ValidatedAccentPhrase<'original> {
    fn new(original: &'original AccentPhrase) -> Result<Self, InvalidQueryErrorKind> {
        let moras = original
            .moras
            .iter()
            .map(ValidatedMora::new)
            .collect::<Result<_, _>>()?;

        let accent = NonZero::new(original.accent).ok_or(InvalidQueryErrorKind::AccentIsZero)?;

        let pause_mora = original
            .pause_mora
            .as_ref()
            .map(ValidatedMora::new)
            .transpose()?;

        Ok(Self {
            moras,
            accent,
            pause_mora,
            is_interrogative: original.is_interrogative,
        })
    }

    fn into_owned(self) -> ValidatedAccentPhrase<'static> {
        let Self {
            moras,
            accent,
            pause_mora,
            is_interrogative,
        } = self;
        let moras = moras.into_iter().map(ValidatedMora::into_owned).collect();
        let pause_mora = pause_mora.map(ValidatedMora::into_owned);
        ValidatedAccentPhrase {
            moras,
            accent,
            pause_mora,
            is_interrogative,
        }
    }
}

impl From<ValidatedAccentPhrase<'_>> for AccentPhrase {
    fn from(
        ValidatedAccentPhrase {
            moras,
            accent,
            pause_mora,
            is_interrogative,
        }: ValidatedAccentPhrase<'_>,
    ) -> Self {
        Self {
            moras: moras.into_iter().map(Into::into).collect(),
            accent: accent.get(),
            pause_mora: pause_mora.map(Into::into),
            is_interrogative,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ValidatedAudioQuery<'original> {
    pub(crate) accent_phrases: Vec<ValidatedAccentPhrase<'original>>,
    pub(crate) speed_scale: f32,
    pub(crate) pitch_scale: f32,
    pub(crate) intonation_scale: f32,
    pub(crate) volume_scale: f32,
    pub(crate) pre_phoneme_length: f32,
    pub(crate) post_phoneme_length: f32,
    pub(crate) output_sampling_rate: SamplingRate,
    pub(crate) output_stereo: bool,
    pub(crate) kana: Option<String>,
}

impl<'original> ValidatedAudioQuery<'original> {
    fn new(original: &'original AudioQuery) -> Result<Self, InvalidQueryErrorKind> {
        let accent_phrases = original
            .accent_phrases
            .iter()
            .map(ValidatedAccentPhrase::new)
            .collect::<Result<_, _>>()?;

        let output_sampling_rate = SamplingRate::new(original.output_sampling_rate).ok_or(
            InvalidQueryErrorKind::InvalidSamplingRate(original.output_sampling_rate),
        )?;

        let AudioQuery {
            speed_scale,
            pitch_scale,
            intonation_scale,
            volume_scale,
            pre_phoneme_length,
            post_phoneme_length,
            output_stereo,
            ..
        } = *original;
        let kana = original.kana.clone();

        Ok(Self {
            accent_phrases,
            speed_scale,
            pitch_scale,
            intonation_scale,
            volume_scale,
            pre_phoneme_length,
            post_phoneme_length,
            output_sampling_rate,
            output_stereo,
            kana,
        })
    }

    pub(crate) fn into_owned(self) -> ValidatedAudioQuery<'static> {
        let Self {
            accent_phrases,
            speed_scale,
            pitch_scale,
            intonation_scale,
            volume_scale,
            pre_phoneme_length,
            post_phoneme_length,
            output_sampling_rate,
            output_stereo,
            kana,
        } = self;
        let accent_phrases = accent_phrases
            .into_iter()
            .map(ValidatedAccentPhrase::into_owned)
            .collect();
        ValidatedAudioQuery {
            accent_phrases,
            speed_scale,
            pitch_scale,
            intonation_scale,
            volume_scale,
            pre_phoneme_length,
            post_phoneme_length,
            output_sampling_rate,
            output_stereo,
            kana,
        }
    }
}

impl From<ValidatedAudioQuery<'_>> for AudioQuery {
    fn from(
        ValidatedAudioQuery {
            accent_phrases,
            speed_scale,
            pitch_scale,
            intonation_scale,
            volume_scale,
            pre_phoneme_length,
            post_phoneme_length,
            output_sampling_rate,
            output_stereo,
            kana,
        }: ValidatedAudioQuery<'_>,
    ) -> Self {
        Self {
            accent_phrases: accent_phrases.into_iter().map(Into::into).collect(),
            speed_scale,
            pitch_scale,
            intonation_scale,
            volume_scale,
            pre_phoneme_length,
            post_phoneme_length,
            output_sampling_rate: output_sampling_rate.get(),
            output_stereo,
            kana,
        }
    }
}

mod sampling_rate {
    use std::num::NonZero;

    use crate::engine::DEFAULT_SAMPLING_RATE;

    #[derive(Clone, Copy, PartialEq, Debug)]
    pub(crate) struct SamplingRate(NonZero<u32>);

    impl SamplingRate {
        pub(super) fn new(n: u32) -> Option<Self> {
            NonZero::new(n)
                .filter(|n| n.get() % DEFAULT_SAMPLING_RATE == 0)
                .map(Self)
        }

        pub(crate) fn get(self) -> u32 {
            self.0.get()
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::{super::super::DEFAULT_SAMPLING_RATE, AudioQuery};

    #[rstest]
    fn it_accepts_json_without_optional_fields() -> anyhow::Result<()> {
        serde_json::from_value::<AudioQuery>(json!({
            "accent_phrases": [
                {
                    "moras": [
                        {
                            "text": "ア",
                            "vowel": "a",
                            "vowel_length": 0.0,
                            "pitch": 0.0
                        }
                    ],
                    "accent": 1
                }
            ],
            "speedScale": 1.0,
            "pitchScale": 0.0,
            "intonationScale": 1.0,
            "volumeScale": 1.0,
            "prePhonemeLength": 0.1,
            "postPhonemeLength": 0.1,
            "outputSamplingRate": DEFAULT_SAMPLING_RATE,
            "outputStereo": false
        }))?;
        Ok(())
    }
}
