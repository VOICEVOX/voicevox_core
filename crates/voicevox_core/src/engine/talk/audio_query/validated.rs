use std::{borrow::Cow, num::NonZero};

use crate::error::{ErrorRepr, InvalidQueryErrorKind};

use super::{super::super::acoustic_feature_extractor::Phoneme, AccentPhrase, AudioQuery, Mora};

pub(crate) use self::sampling_rate::SamplingRate;

impl Mora {
    /// この構造体をバリデートする。
    ///
    /// # Errors
    ///
    /// 次のうちどれかを満たすなら[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
    ///
    /// - [`consonant`]と[`consonant_length`]の有無が不一致。
    /// - [`consonant`]もしくは[`vowel`]が音素として不正。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`consonant`]: Self::consonant
    /// [`consonant_length`]: Self::consonant_length
    /// [`vowel`]: Self::vowel
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

impl AccentPhrase {
    /// この構造体をバリデートする。
    ///
    /// # Errors
    ///
    /// 次のうちどれかを満たすなら[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
    ///
    /// - [`moras`]もしくは[`pause_mora`]の要素のうちいずれかが[不正]。
    /// - [`accent`]が`0`。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`moras`]: Self::moras
    /// [`pause_mora`]: Self::pause_mora
    /// [`accent`]: Self::accent
    /// [不正]: Mora::validate
    #[cfg_attr(doc, doc(alias = "voicevox_validate_accent_phrases"))]
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

impl AudioQuery {
    /// この構造体をバリデートする。
    ///
    /// # Errors
    ///
    /// 次のうちどれかを満たすなら[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
    ///
    /// - [`accent_phrases`]の要素のうちいずれかが[不正]。
    /// - [`output_sampling_rate`]が`24000`の倍数ではない、もしくは`0` (将来的に解消予定。cf. [#762])。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`accent_phrases`]: Self::accent_phrases
    /// [`output_sampling_rate`]: Self::output_sampling_rate
    /// [不正]: AccentPhrase::validate
    /// [#762]: https://github.com/VOICEVOX/voicevox_core/issues/762
    #[cfg_attr(doc, doc(alias = "voicevox_audio_query_validate"))]
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
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct ValidatedMora<'original> {
    pub(crate) text: Cow<'original, str>,
    pub(crate) consonant: Option<LengthedPhoneme>,
    pub(crate) vowel: LengthedPhoneme,
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
pub(crate) struct LengthedPhoneme {
    pub(crate) phoneme: Phoneme,
    pub(crate) length: f32,
}

impl LengthedPhoneme {
    fn new(phoneme: &str, length: f32) -> Result<Self, InvalidQueryErrorKind> {
        let phoneme = phoneme
            .parse()
            .map_err(|_| InvalidQueryErrorKind::InvalidPhoneme(phoneme.to_owned()))?;
        Ok(Self { phoneme, length })
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
