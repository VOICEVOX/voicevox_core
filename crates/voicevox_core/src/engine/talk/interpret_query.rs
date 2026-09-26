//! [`AudioQuery`]から特徴量を取り出す処理を集めたもの。

use std::{num::Saturating, ops::Add};

use itertools::chain;
use typed_floats::{NonNaNFinite, PositiveFinite, tf32};

use crate::{
    AccentPhrase, AudioQuery, Mora,
    numerics::{non_nan_finite_f32, positive_finite_f32},
};

use super::{
    super::{
        DEFAULT_SAMPLING_RATE, PhonemeCode,
        acoustic_feature_extractor::{MoraTail, OptionalConsonant},
        talk::{LengthedPhoneme, ValidatedAccentPhrase, ValidatedAudioQuery, ValidatedMora},
    },
    full_context_label::mora_to_text,
};

pub const DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK: bool = true;

const FIX_VOWEL_LENGTH: PositiveFinite<f32> = positive_finite_f32!(0.15);

pub(crate) fn initial_process<'query>(
    accent_phrases: &[ValidatedAccentPhrase<'query>],
) -> (Vec<ValidatedMora<'query>>, Vec<PhonemeCode>) {
    let flatten_moras = to_flatten_moras(accent_phrases);

    let mut phoneme_data_list = vec![PhonemeCode::MorablePau];
    for mora in flatten_moras.iter() {
        if let Some(consonant) = &mora.consonant {
            phoneme_data_list.push(consonant.phoneme.into())
        }
        phoneme_data_list.push(mora.vowel.phoneme.clone().into());
    }
    phoneme_data_list.push(PhonemeCode::MorablePau);

    return (flatten_moras, phoneme_data_list);

    fn to_flatten_moras<'query>(
        accent_phrases: &[ValidatedAccentPhrase<'query>],
    ) -> Vec<ValidatedMora<'query>> {
        let mut flatten_moras = Vec::new();

        for ValidatedAccentPhrase {
            moras, pause_mora, ..
        } in accent_phrases
        {
            for mora in moras {
                flatten_moras.push(mora.clone());
            }
            if let Some(pause_mora) = pause_mora {
                flatten_moras.push(pause_mora.clone());
            }
        }

        flatten_moras
    }
}

pub(crate) fn split_mora(
    phoneme_list: &[PhonemeCode],
) -> (Vec<OptionalConsonant>, Vec<MoraTail>, Vec<i64>) {
    let mut vowel_phoneme_list = Vec::new();
    let mut vowel_indexes = Vec::new();
    for (i, phoneme) in phoneme_list.iter().enumerate() {
        if let Ok(mora_tail) = (*phoneme).try_into() {
            vowel_phoneme_list.push(mora_tail);
            vowel_indexes.push(i as i64);
        }
    }

    let mut consonant_phoneme_list = vec![OptionalConsonant::None];
    for i in 0..(vowel_indexes.len() - 1) {
        let prev = vowel_indexes[i];
        let next = vowel_indexes[i + 1];
        if next - prev == 1 {
            consonant_phoneme_list.push(OptionalConsonant::None);
        } else {
            consonant_phoneme_list.push(
                phoneme_list[next as usize - 1]
                    .try_into()
                    .expect("`OptionalConsonant` and `MoraTail` should be exclusive"),
            );
        }
    }

    (consonant_phoneme_list, vowel_phoneme_list, vowel_indexes)
}

impl AudioQuery {
    /// 音声の総フレーム数を算出する。
    ///
    /// 音声の秒数は、フレーム数を[`FRAME_RATE`]で割った値で表せる。
    ///
    /// 算出方法は以下の通り。
    ///
    /// 1. 32-bit浮動小数点数の値として存在する以下の秒数を集める。
    ///     - [`AudioQuery::pre_phoneme_length`]
    ///     - [`AudioQuery::accent_phrases`]の要素ごとに
    ///         - [`AccentPhrase::moras`]の要素ごとに
    ///             - [`Mora::consonant_length`]
    ///             - [`Mora::vowel_length`]
    ///         - [`enable_interrogative_upspeak`]かつ[`AccentPhrase::is_interrogative`]かつ[`AccentPhrase::moras`]の最後の[`Mora::pitch`]が`0.0`以外のとき、`0.15`秒
    ///         - [`AccentPhrase::pause_mora`]の[`Mora::consonant_length`]（通常はない）
    ///         - [`AccentPhrase::pause_mora`]の[`Mora::vowel_length`]
    ///     - [`AudioQuery::post_phoneme_length`]
    /// 2. それぞれの秒数を`secs`として、対応するフレーム長を<code>((secs * [FRAME_RATE]).[round_ties_even()] / [speed_scale]).[round_ties_even()]</code>として算出する。
    /// 3. 各フレーム長を足し合わせる。
    ///
    /// # Caveats
    ///
    /// `AudioQuery`に対応する音声の長さは将来的に変わる可能性がある。例えば、秒数を64-bit浮動小数点数として解釈しているVOICEVOX
    /// ENGINEと挙動を揃える可能性がある。
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use pollster::FutureExt as _;
    /// # use voicevox_core::{__internal::doctest_fixtures::IntoBlocking as _, StyleId};
    /// #
    /// # const WHATEVER_STYLE1: StyleId = StyleId(0);
    /// # const WHATEVER_STYLE2: StyleId = StyleId(302);
    /// #
    /// # let synth =
    /// #     voicevox_core::__internal::doctest_fixtures::synthesizer_with_sample_voice_model(
    /// #         test_util::SAMPLE_VOICE_MODEL_FILE_PATH,
    /// #         test_util::ONNXRUNTIME_DYLIB_PATH,
    /// #         test_util::OPEN_JTALK_DIC_DIR,
    /// #     )
    /// #     .block_on()?
    /// #     .into_blocking();
    /// #
    /// let query =
    ///     &synth.create_audio_query("こんにちは、音声合成の世界へようこそ？", WHATEVER_STYLE1)?;
    /// let audio = synth
    ///     .create_audio_feature(query, WHATEVER_STYLE2)
    ///     .perform()?;
    ///
    /// assert_eq!(audio.frame_length(), query.frame_length().calculate().0);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # use voicevox_core::AudioQuery;
    /// #
    /// let mut query = AudioQuery::from(vec![]);
    /// query.pre_phoneme_length = typed_floats::as_const!(PositiveFinite, f32, 3.3);
    /// query.post_phoneme_length = typed_floats::as_const!(PositiveFinite, f32, 4.4);
    /// query.speed_scale = typed_floats::as_const!(PositiveFinite, f32, 1.2);
    ///
    /// assert_eq!(
    ///     // `speed_scale`, `pre_phoneme_length`
    ///     to_frame_length(3.3, 1.2)
    ///         // `speed_scale`, `consonant_length`, `vowel_length`, `is_interrogative`
    ///         + 0
    ///         // `speed_scale`, `post_phoneme_length`
    ///         + to_frame_length(4.4, 1.2),
    ///     query.frame_length().calculate().0,
    /// );
    ///
    /// fn to_frame_length(secs: f32, speed_scale: f32) -> usize {
    ///     ((secs * 93.75).round_ties_even() / speed_scale).round_ties_even() as _
    /// }
    /// ```
    ///
    /// ```
    /// # use voicevox_core::AudioQuery;
    /// #
    /// use typed_floats::tf32;
    ///
    /// let mut query = AudioQuery::from(vec![]);
    /// query.speed_scale = tf32::MIN_POSITIVE.into();
    /// assert_eq!(usize::MAX, query.frame_length().calculate().0);
    /// ```
    ///
    /// [`FRAME_RATE`]: crate::AudioFeature::FRAME_RATE
    /// [`enable_interrogative_upspeak`]: AudioQueryFrameLength::enable_interrogative_upspeak
    /// [FRAME_RATE]: crate::AudioFeature::FRAME_RATE
    /// [round_ties_even()]: f64::round_ties_even
    /// [speed_scale]: Self::speed_scale
    pub fn frame_length(&self) -> AudioQueryFrameLength<'_> {
        AudioQueryFrameLength {
            audio_query: self,
            enable_interrogative_upspeak: DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
        }
    }
}

/// [`AudioQuery::frame_length`]のビルダー。
#[must_use = "this is a builder. it does nothing until `calculate`d"]
#[derive(Debug)]
pub struct AudioQueryFrameLength<'a> {
    audio_query: &'a AudioQuery,
    enable_interrogative_upspeak: bool,
}

impl AudioQueryFrameLength<'_> {
    pub fn enable_interrogative_upspeak(mut self, enable_interrogative_upspeak: bool) -> Self {
        self.enable_interrogative_upspeak = enable_interrogative_upspeak;
        self
    }

    /// 音声の総フレーム数を算出する。
    ///
    /// 詳細は[`AudioQuery::frame_length`]を参照。
    pub fn calculate(self) -> Saturating<usize> {
        return chain!(
            [self.audio_query.pre_phoneme_length],
            self.audio_query.accent_phrases.iter().flat_map(
                |AccentPhrase {
                     moras,
                     pause_mora,
                     is_interrogative,
                     ..
                 }| chain!(
                    lengths(moras),
                    (self.enable_interrogative_upspeak
                        && *is_interrogative
                        && moras.last().is_some_and(|Mora { pitch, .. }| *pitch != 0.0))
                    .then_some(FIX_VOWEL_LENGTH),
                    lengths(pause_mora.as_ref()),
                ),
            ),
            [self.audio_query.post_phoneme_length],
        )
        .map(|length| to_frame_length(length.get(), self.audio_query.speed_scale.get()))
        .map(Saturating)
        .fold(Saturating(0), Add::add); // TODO: Rust 1.91以降なら`Sum`を使える

        fn lengths<'a>(
            moras: impl IntoIterator<Item = &'a Mora>,
        ) -> impl Iterator<Item = PositiveFinite<f32>> {
            moras.into_iter().flat_map(
                |&Mora {
                     consonant_length,
                     vowel_length,
                     ..
                 }| itertools::chain(consonant_length, [vowel_length]),
            )
        }
    }
}

pub(crate) struct DecoderFeature {
    pub(crate) f0: Vec<f32>,
    pub(crate) phoneme: Vec<[f32; PhonemeCode::num_phoneme()]>,
}

impl ValidatedAudioQuery<'_> {
    pub(crate) fn decoder_feature(&self, enable_interrogative_upspeak: bool) -> DecoderFeature {
        let ValidatedAudioQuery {
            accent_phrases,
            speed_scale,
            pitch_scale,
            intonation_scale,
            pre_phoneme_length,
            post_phoneme_length,
            ..
        } = self;

        // FIXME: 可能な範囲でtyped_floatsを取り回し続けるべきではないか？
        let speed_scale = f32::from(*speed_scale);
        let pitch_scale = f32::from(*pitch_scale);
        let intonation_scale = f32::from(*intonation_scale);
        let pre_phoneme_length = f32::from(*pre_phoneme_length);
        let post_phoneme_length = f32::from(*post_phoneme_length);

        let accent_phrases = if enable_interrogative_upspeak {
            &adjust_interrogative_accent_phrases(accent_phrases)
        } else {
            accent_phrases
        };

        let (flatten_moras, phoneme_data_list) = initial_process(accent_phrases);

        let mut phoneme_length_list = vec![pre_phoneme_length];
        let mut f0_list = vec![0.];
        let mut voiced_list = vec![false];
        {
            let mut sum_of_f0_bigger_than_zero = 0.;
            let mut count_of_f0_bigger_than_zero = 0;

            for ValidatedMora {
                consonant,
                vowel,
                pitch,
                ..
            } in flatten_moras
            {
                if let Some(consonant) = consonant {
                    phoneme_length_list.push(consonant.length.into());
                }
                phoneme_length_list.push(vowel.length.into());

                let f0_single = f32::from(pitch) * 2.0_f32.powf(pitch_scale);
                f0_list.push(f0_single);

                let bigger_than_zero = f0_single > 0.;
                voiced_list.push(bigger_than_zero);

                if bigger_than_zero {
                    sum_of_f0_bigger_than_zero += f0_single;
                    count_of_f0_bigger_than_zero += 1;
                }
            }
            phoneme_length_list.push(post_phoneme_length);
            f0_list.push(0.);
            voiced_list.push(false);
            let mean_f0 = sum_of_f0_bigger_than_zero / (count_of_f0_bigger_than_zero as f32);

            if !mean_f0.is_nan() {
                for i in 0..f0_list.len() {
                    if voiced_list[i] {
                        f0_list[i] = (f0_list[i] - mean_f0) * intonation_scale + mean_f0;
                    }
                }
            }
        }

        let (_, _, vowel_indexes) = split_mora(&phoneme_data_list);

        let mut phoneme = Vec::new();
        let mut f0: Vec<f32> = Vec::new();
        {
            let mut sum_of_phoneme_length = 0;
            let mut count_of_f0 = 0;
            let mut vowel_indexes_index = 0;

            for (i, phoneme_length) in phoneme_length_list.iter().enumerate() {
                let phoneme_length = to_frame_length(*phoneme_length, speed_scale);
                let phoneme_id = usize::from(phoneme_data_list[i]);

                for _ in 0..phoneme_length {
                    let mut phonemes_vec = [0.; _];
                    phonemes_vec[phoneme_id] = 1.;
                    phoneme.push(phonemes_vec)
                }
                sum_of_phoneme_length += phoneme_length;

                if i as i64 == vowel_indexes[vowel_indexes_index] {
                    for _ in 0..sum_of_phoneme_length {
                        f0.push(f0_list[count_of_f0]);
                    }
                    count_of_f0 += 1;
                    sum_of_phoneme_length = 0;
                    vowel_indexes_index += 1;
                }
            }
        }
        return DecoderFeature { f0, phoneme };

        fn adjust_interrogative_accent_phrases<'query>(
            accent_phrases: &[ValidatedAccentPhrase<'query>],
        ) -> Vec<ValidatedAccentPhrase<'query>> {
            accent_phrases
                .iter()
                .map(|accent_phrase| ValidatedAccentPhrase {
                    moras: adjust_interrogative_moras(accent_phrase),
                    ..accent_phrase.clone()
                })
                .collect()
        }

        fn adjust_interrogative_moras<'query>(
            ValidatedAccentPhrase {
                moras,
                is_interrogative,
                ..
            }: &ValidatedAccentPhrase<'query>,
        ) -> Vec<ValidatedMora<'query>> {
            if *is_interrogative && !moras.is_empty() {
                let last_mora = moras.last().unwrap();
                if last_mora.pitch != 0.0 {
                    let mut new_moras = Vec::with_capacity(moras.len() + 1);
                    new_moras.extend_from_slice(moras.as_slice());
                    let interrogative_mora = make_interrogative_mora(last_mora);
                    new_moras.push(interrogative_mora);
                    return new_moras;
                }
            }
            moras.clone()
        }

        fn make_interrogative_mora<'query>(
            last_mora: &ValidatedMora<'query>,
        ) -> ValidatedMora<'query> {
            const ADJUST_PITCH: NonNaNFinite<f32> = non_nan_finite_f32!(0.3);
            const MAX_PITCH: NonNaNFinite<f32> = non_nan_finite_f32!(6.5);

            let pitch = NonNaNFinite::try_from(last_mora.pitch + ADJUST_PITCH)
                .unwrap_or_else(|_| tf32::MAX.into())
                .min(MAX_PITCH);

            ValidatedMora {
                text: mora_to_text(None, &last_mora.vowel.phoneme.to_string()).into(),
                consonant: None,
                vowel: LengthedPhoneme {
                    phoneme: last_mora.vowel.phoneme.clone(),
                    length: FIX_VOWEL_LENGTH,
                },
                pitch,
            }
        }
    }
}

fn to_frame_length(secs: f32, speed_scale: f32) -> usize {
    // VOICEVOX ENGINEと挙動を合わせるため、四捨五入ではなく偶数丸めをする
    //
    // https://github.com/VOICEVOX/voicevox_engine/issues/552
    const RATE: f32 = DEFAULT_SAMPLING_RATE as f32 / 256.;
    ((secs * RATE).round_ties_even() / speed_scale).round_ties_even() as _
}
