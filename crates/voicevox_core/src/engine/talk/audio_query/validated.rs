use std::{
    borrow::Cow,
    num::{FpCategory, NonZero},
    str::FromStr,
};

use duplicate::duplicate_item;
use tracing::warn;

use crate::error::{InvalidQueryError, InvalidQueryErrorSource};

use super::{
    super::super::{
        acoustic_feature_extractor::{Consonant, NonConsonant},
        sampling_rate::SamplingRate,
        validate::Validate as _,
        DEFAULT_SAMPLING_RATE,
    },
    AccentPhrase, AudioQuery, Mora,
};

impl Mora {
    /// この構造体をバリデートする。
    ///
    /// # Errors
    ///
    /// 次のうちどれかを満たすなら[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
    ///
    /// - [`consonant`]と[`consonant_length`]の有無が不一致。
    /// - [`consonant`]が子音以外の音素であるか、もしくは[`Phoneme`]として不正。
    /// - [`vowel`]が子音であるか、もしくは[`Phoneme`]として不正。
    ///
    /// # Warnings
    ///
    /// 次の状態に対しては[`WARN`]レベルのログを出す。将来的にはエラーになる予定。
    ///
    /// - [`consonant_length`]がNaN、infinity、もしくは負。
    /// - [`vowel_length`]がNaN、infinity、もしくは負。
    /// - [`pitch`]がNaNもしくは±infinity。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`Phoneme`]: crate::Phoneme
    /// [`WARN`]: tracing::Level::WARN
    /// [`consonant`]: Self::consonant
    /// [`consonant_length`]: Self::consonant_length
    /// [`vowel`]: Self::vowel
    /// [`vowel_length`]: Self::vowel_length
    /// [`pitch`]: Self::pitch
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
    /// この構造体をバリデートする。
    ///
    /// # Errors
    ///
    /// 次のうちどれかを満たすなら[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
    ///
    /// - [`moras`]もしくは[`pause_mora`]の要素のうちいずれかが[不正]。
    /// - [`accent`]が`0`。
    ///
    /// # Warnings
    ///
    /// 次の状態に対しては[`WARN`]レベルのログを出す。将来的にはエラーになる予定。
    ///
    /// - [`moras`]もしくは[`pause_mora`]の要素のうちいずれかが、警告が出る状態。
    /// - [`accent`]が[`moras`]の数を超過している。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`WARN`]: tracing::Level::WARN
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
    /// この構造体をバリデートする。
    ///
    /// # Errors
    ///
    /// 次のうちどれかを満たすなら[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
    ///
    /// - [`accent_phrases`]の要素のうちいずれかが[不正]。
    /// - [`output_sampling_rate`]が`24000`の倍数ではない、もしくは`0` (将来的に解消予定。cf. [#762])。
    ///
    /// 次の状態に対しては[`WARN`]レベルのログを出す。将来的にはエラーになる予定。
    ///
    /// - [`accent_phrases`]の要素のうちいずれかが警告が出る状態。
    /// - [`speed_scale`]がNaN、infinity、もしくは負。
    /// - [`pitch_scale`]がNaNもしくは±infinity。
    /// - [`intonation_scale`]がNaNもしくは±infinity。
    /// - [`volume_scale`]がNaN、infinity、もしくは負。
    /// - [`pre_phoneme_length`]がNaN、infinity、もしくは負。
    /// - [`post_phoneme_length`]がNaN、infinity、もしくは負。
    /// - [`output_sampling_rate`]が`24000`以外の値（エラーと同様将来的に解消予定）。
    ///
    /// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
    /// [`WARN`]: tracing::Level::WARN
    /// [`accent_phrases`]: Self::accent_phrases
    /// [`speed_scale`]: Self::speed_scale
    /// [`pitch_scale`]: Self::pitch_scale
    /// [`intonation_scale`]: Self::intonation_scale
    /// [`volume_scale`]: Self::volume_scale
    /// [`pre_phoneme_length`]: Self::pre_phoneme_length
    /// [`post_phoneme_length`]: Self::post_phoneme_length
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

macro_rules! warn_for_non_finite {
    ($v:ident $(,)?) => {
        match $v.classify() {
            FpCategory::Nan => warn!("`{}` should not be NaN", stringify!($v)),
            FpCategory::Infinite => warn!("`{}` should not be infinite", stringify!($v)),
            FpCategory::Zero | FpCategory::Subnormal | FpCategory::Normal => {}
        }
    };
}

macro_rules! warn_for_non_positive_finite {
    ($v:ident $(,)?) => {
        match $v.classify() {
            FpCategory::Nan | FpCategory::Infinite => warn_for_non_finite!($v),
            FpCategory::Zero | FpCategory::Subnormal | FpCategory::Normal => {
                if $v.is_sign_negative() {
                    warn!("`{}` should not be negative", stringify!($v));
                }
            }
        }
    };
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct ValidatedMora<'original> {
    pub(crate) text: Cow<'original, str>,
    pub(crate) consonant: Option<LengthedPhoneme<Consonant>>,
    pub(crate) vowel: LengthedPhoneme<NonConsonant>,
    pub(crate) pitch: f32,
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

        if let Some(consonant_length) = consonant_length {
            warn_for_non_positive_finite!(consonant_length);
        }
        warn_for_non_positive_finite!(vowel_length);
        warn_for_non_finite!(pitch);

        let consonant = match (consonant, consonant_length) {
            (Some(phoneme), Some(length)) => {
                Some(LengthedPhoneme::new(phoneme, length).map_err(|source| {
                    error(InvalidQueryErrorSource::InvalidFields {
                        fields: "`consonant`".to_owned(),
                        source: source.into(),
                    })
                })?)
            }
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

        let vowel = LengthedPhoneme::new(vowel, vowel_length).map_err(|source| {
            error(InvalidQueryErrorSource::InvalidFields {
                fields: "`vowel`".to_owned(),
                source: source.into(),
            })
        })?;

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
                .map(|LengthedPhoneme { phoneme, .. }| phoneme.to_string()),
            consonant_length: consonant.map(|LengthedPhoneme { length, .. }| length),
            vowel: vowel.phoneme.to_string(),
            vowel_length: vowel.length,
            pitch,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct LengthedPhoneme<P> {
    pub(crate) phoneme: P,
    pub(crate) length: f32,
}

impl<P> LengthedPhoneme<P> {
    fn new(phoneme: &str, length: f32) -> Result<Self, InvalidQueryError>
    where
        P: FromStrWithInnerError,
    {
        let phoneme = FromStrWithInnerError::from_str_with_inner_error(phoneme)?;
        Ok(Self { phoneme, length })
    }
}

trait FromStrWithInnerError: FromStr {
    fn from_str_with_inner_error(s: &str) -> Result<Self, InvalidQueryError>;
}

#[duplicate_item(
    T;
    [ Consonant ];
    [ NonConsonant ];
)]
impl FromStrWithInnerError for T {
    fn from_str_with_inner_error(s: &str) -> Result<Self, InvalidQueryError> {
        Self::from_str_with_inner_error(s)
    }
}

impl<P> From<P> for LengthedPhoneme<P> {
    fn from(phoneme: P) -> Self {
        Self {
            phoneme,
            length: 0.,
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
        let is_interrogative = *is_interrogative;

        if *accent > moras.len() {
            warn!("`accent` should not exceed the number of `moras`");
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

        let accent = NonZero::new(*accent).ok_or_else(|| {
            assert_eq!(0, *accent);
            error(InvalidQueryErrorSource::InvalidFields {
                fields: "`accent`".to_owned(),
                source: InvalidQueryError {
                    what: "アクセント位置",
                    value: Some(Box::new(0usize)),
                    source: Some(InvalidQueryErrorSource::IsZero),
                }
                .into(),
            })
        })?;

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
        let output_stereo = *output_stereo;

        warn_for_non_positive_finite!(speed_scale);
        warn_for_non_finite!(pitch_scale);
        warn_for_non_finite!(intonation_scale);
        warn_for_non_positive_finite!(volume_scale);
        warn_for_non_positive_finite!(pre_phoneme_length);
        warn_for_non_positive_finite!(post_phoneme_length);
        if *output_sampling_rate != DEFAULT_SAMPLING_RATE {
            warn!("`output_sampling_rate` should be `DEFAULT_SAMPLING_RATE`");
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

        let output_sampling_rate = SamplingRate::new_(*output_sampling_rate).map_err(|source| {
            error(InvalidQueryErrorSource::InvalidFields {
                fields: "`output_sampling_rate`/`outputSamplingRate`".to_owned(),
                source: source.into(),
            })
        })?;

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
            output_sampling_rate: output_sampling_rate.get().get(),
            output_stereo,
            kana,
        }
    }
}
