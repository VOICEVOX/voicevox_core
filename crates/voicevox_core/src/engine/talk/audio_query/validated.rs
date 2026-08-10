use std::{borrow::Cow, num::NonZero};

use tracing::warn;
use typed_floats::{NonNaNFinite, PositiveFinite, tf32};

use crate::error::{InvalidQueryError, InvalidQueryErrorSource};

use super::{
    super::super::{
        acoustic_feature_extractor::{Consonant, NonConsonant},
        sampling_rate::SamplingRate,
        validate::Validate as _,
    },
    AccentPhrase, AudioQuery, Mora,
};

impl Mora {
    /// この構造体が不正であるときエラーを返す。
    ///
    /// # Errors
    ///
    /// この構造体が不正であるとき[`ErrorKind::InvalidQuery`]を表わすエラーを返す。不正であるとは、以下の条件を満たすことである。
    ///
    /// - [`consonant`]と[`consonant_length`]の有無が不一致。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`consonant`]: Self::consonant
    /// [`consonant_length`]: Self::consonant_length
    #[cfg_attr(doc, doc(alias = "voicevox_mora_validate"))]
    pub fn validate(&self) -> crate::Result<()> {
        self.to_validated().map(|_| ())
    }

    // TODO: この層を破壊
    fn to_validated(&self) -> crate::Result<ValidatedMora<'_>> {
        ValidatedMora::new(self).map_err(Into::into)
    }
}

impl AccentPhrase {
    /// この構造体が不正であるときエラーを返す。
    ///
    /// # Errors
    ///
    /// この構造体が不正であるとき[`ErrorKind::InvalidQuery`]を表わすエラーを返す。不正であるとは、以下のいずれかの条件を満たすことである。
    ///
    /// - [`moras`]もしくは[`pause_mora`]の要素のうちいずれかが[不正]。
    /// - [`accent`]が[`moras`]の数を超過している。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`moras`]: Self::moras
    /// [`pause_mora`]: Self::pause_mora
    /// [`accent`]: Self::accent
    /// [不正]: Mora::validate
    #[cfg_attr(doc, doc(alias = "voicevox_accent_phrase_validate"))]
    pub fn validate(&self) -> crate::Result<()> {
        self.to_validated().map(|_| ())
    }

    // TODO: この層を破壊
    pub(crate) fn to_validated(&self) -> crate::Result<ValidatedAccentPhrase<'_>> {
        ValidatedAccentPhrase::new(self).map_err(Into::into)
    }
}

impl AudioQuery {
    /// この構造体が不正であるときエラーを返す。
    ///
    /// # Errors
    ///
    /// この構造体が不正であるとき[`ErrorKind::InvalidQuery`]を表わすエラーを返す。不正であるとは、以下の条件を満たすことである。
    ///
    /// - [`accent_phrases`]の要素のうちいずれかが[不正]。
    ///
    /// # Warnings
    ///
    /// 次の状態に対しては[`WARN`]レベルのログを出す。将来的にはエラーになる予定。
    ///
    /// - [`output_sampling_rate`]が`24000`以外の値（将来的に解消予定。cf. [#762]）。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`WARN`]: tracing::Level::WARN
    /// [`accent_phrases`]: Self::accent_phrases
    /// [`output_sampling_rate`]: Self::output_sampling_rate
    /// [不正]: AccentPhrase::validate
    /// [#762]: https://github.com/VOICEVOX/voicevox_core/issues/762
    #[cfg_attr(doc, doc(alias = "voicevox_audio_query_validate"))]
    pub fn validate(&self) -> crate::Result<()> {
        self.to_validated().map(|_| ())
    }

    // TODO: この層を破壊
    pub(crate) fn to_validated(&self) -> crate::Result<ValidatedAudioQuery<'_>> {
        ValidatedAudioQuery::new(self).map_err(Into::into)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct ValidatedMora<'original> {
    pub(crate) text: Cow<'original, str>,
    pub(crate) consonant: Option<LengthedPhoneme<Consonant>>,
    pub(crate) vowel: LengthedPhoneme<NonConsonant>,
    pub(crate) pitch: NonNaNFinite<f32>,
}

impl<'original> ValidatedMora<'original> {
    fn new(original: &'original Mora) -> Result<Self, InvalidQueryError> {
        let Mora {
            text,
            consonant,
            consonant_length,
            vowel,
            vowel_length,
            pitch,
        } = original;
        let consonant_length = *consonant_length;
        let vowel_length = *vowel_length;
        let pitch = *pitch;

        let consonant = match (consonant, consonant_length) {
            (Some(phoneme), Some(length)) => Some(LengthedPhoneme {
                phoneme: *phoneme,
                length,
            }),
            (None, None) => None,
            (Some(_), None) | (None, Some(_)) => {
                return Err(error(InvalidQueryErrorSource::InvalidFields {
                    fields: "`consonant`と`consonant_length`".to_owned(),
                    source: InvalidQueryError {
                        what: "組み合わせ",
                        value: None,
                        source: InvalidQueryErrorSource::PartiallyPresent.into(),
                    }
                    .into(),
                }));
            }
        };

        let vowel = LengthedPhoneme {
            phoneme: vowel.clone(),
            length: vowel_length,
        };

        let text = text.into();

        return Ok(Self {
            text,
            consonant,
            vowel,
            pitch,
        });

        fn error(source: InvalidQueryErrorSource) -> InvalidQueryError {
            InvalidQueryError {
                what: Mora::NAME,
                value: None,
                source: Some(source),
            }
        }
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
                .map(|LengthedPhoneme { phoneme, .. }| *phoneme),
            consonant_length: consonant.map(|LengthedPhoneme { length, .. }| length),
            vowel: vowel.phoneme,
            vowel_length: vowel.length,
            pitch,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct LengthedPhoneme<P> {
    pub(crate) phoneme: P,
    pub(crate) length: PositiveFinite<f32>,
}

impl<P> From<P> for LengthedPhoneme<P> {
    fn from(phoneme: P) -> Self {
        Self {
            phoneme,
            length: tf32::ZERO,
        }
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
    fn new(original: &'original AccentPhrase) -> Result<Self, InvalidQueryError> {
        let AccentPhrase {
            moras,
            accent,
            pause_mora,
            is_interrogative,
        } = original;
        let accent = *accent;
        let is_interrogative = *is_interrogative;

        if accent.get() > moras.len() {
            return Err(error(InvalidQueryErrorSource::InvalidFields {
                fields: "`moras`と`accent`".to_owned(),
                source: InvalidQueryError {
                    what: "組み合わせ",
                    value: None,
                    source: InvalidQueryErrorSource::TooLargeAccent.into(),
                }
                .into(),
            }));
        }

        let moras = moras
            .iter()
            .enumerate()
            .map(|(i, mora)| {
                ValidatedMora::new(mora).map_err(|source| {
                    error(InvalidQueryErrorSource::InvalidFields {
                        fields: format!("moras[{i}]"),
                        source: source.into(),
                    })
                })
            })
            .collect::<Result<_, _>>()?;

        let pause_mora = pause_mora
            .as_ref()
            .map(ValidatedMora::new)
            .transpose()
            .map_err(|source| {
                error(InvalidQueryErrorSource::InvalidFields {
                    fields: "pause_mora".to_owned(),
                    source: source.into(),
                })
            })?;

        return Ok(Self {
            moras,
            accent,
            pause_mora,
            is_interrogative,
        });

        fn error(source: InvalidQueryErrorSource) -> InvalidQueryError {
            InvalidQueryError {
                what: AccentPhrase::NAME,
                value: None,
                source: Some(source),
            }
        }
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
            accent,
            pause_mora: pause_mora.map(Into::into),
            is_interrogative,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ValidatedAudioQuery<'original> {
    pub(crate) accent_phrases: Vec<ValidatedAccentPhrase<'original>>,
    pub(crate) speed_scale: PositiveFinite<f32>,
    pub(crate) pitch_scale: NonNaNFinite<f32>,
    pub(crate) intonation_scale: NonNaNFinite<f32>,
    pub(crate) volume_scale: PositiveFinite<f32>,
    pub(crate) pre_phoneme_length: PositiveFinite<f32>,
    pub(crate) post_phoneme_length: PositiveFinite<f32>,
    pub(crate) output_sampling_rate: SamplingRate,
    pub(crate) output_stereo: bool,
    pub(crate) kana: Option<String>,
}

impl<'original> ValidatedAudioQuery<'original> {
    fn new(original: &'original AudioQuery) -> Result<Self, InvalidQueryError> {
        let AudioQuery {
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
        } = original;
        let speed_scale = *speed_scale;
        let pitch_scale = *pitch_scale;
        let intonation_scale = *intonation_scale;
        let volume_scale = *volume_scale;
        let pre_phoneme_length = *pre_phoneme_length;
        let post_phoneme_length = *post_phoneme_length;
        let output_sampling_rate = *output_sampling_rate;
        let output_stereo = *output_stereo;

        if output_sampling_rate != SamplingRate::default() {
            warn!("`output_sampling_rate` should be `DEFAULT_SAMPLING_RATE`"); // FIXME: `{}`を忘れてる
        }

        let accent_phrases = accent_phrases
            .iter()
            .enumerate()
            .map(|(i, accent_phrase)| {
                ValidatedAccentPhrase::new(accent_phrase).map_err(|source| {
                    error(InvalidQueryErrorSource::InvalidFields {
                        fields: format!("`accent_phrases[{i}]`"),
                        source: source.into(),
                    })
                })
            })
            .collect::<Result<_, _>>()?;

        let kana = kana.clone();

        return Ok(Self {
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
        });

        fn error(source: InvalidQueryErrorSource) -> InvalidQueryError {
            InvalidQueryError {
                what: AudioQuery::NAME,
                value: None,
                source: Some(source),
            }
        }
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
            output_sampling_rate,
            output_stereo,
            kana,
        }
    }
}
