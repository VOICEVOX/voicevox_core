//! [`AudioQuery`]から特徴量を取り出す処理を集めたもの。

use super::{
    super::{
        acoustic_feature_extractor::{MoraTail, OptionalConsonant},
        talk::{LengthedPhoneme, ValidatedAccentPhrase, ValidatedAudioQuery, ValidatedMora},
        PhonemeCode, DEFAULT_SAMPLING_RATE,
    },
    full_context_label::mora_to_text,
};

pub(crate) fn initial_process<'query>(
    accent_phrases: &[ValidatedAccentPhrase<'query>],
) -> (Vec<ValidatedMora<'query>>, Vec<PhonemeCode>) {
    let flatten_moras = to_flatten_moras(accent_phrases);

    let mut phoneme_data_list = vec![PhonemeCode::MorablePau];
    for mora in flatten_moras.iter() {
        if let Some(consonant) = &mora.consonant {
            phoneme_data_list.push(consonant.phoneme.clone().into_owned().into())
        }
        phoneme_data_list.push(mora.vowel.phoneme.clone().into_owned().into());
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

        let accent_phrases = if enable_interrogative_upspeak {
            &adjust_interrogative_accent_phrases(accent_phrases)
        } else {
            accent_phrases
        };

        let (flatten_moras, phoneme_data_list) = initial_process(accent_phrases);

        let mut phoneme_length_list = vec![*pre_phoneme_length];
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
                    phoneme_length_list.push(consonant.length);
                }
                phoneme_length_list.push(vowel.length);

                let f0_single = pitch * 2.0_f32.powf(*pitch_scale);
                f0_list.push(f0_single);

                let bigger_than_zero = f0_single > 0.;
                voiced_list.push(bigger_than_zero);

                if bigger_than_zero {
                    sum_of_f0_bigger_than_zero += f0_single;
                    count_of_f0_bigger_than_zero += 1;
                }
            }
            phoneme_length_list.push(*post_phoneme_length);
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
            const RATE: f32 = DEFAULT_SAMPLING_RATE as f32 / 256.;
            let mut sum_of_phoneme_length = 0;
            let mut count_of_f0 = 0;
            let mut vowel_indexes_index = 0;

            for (i, phoneme_length) in phoneme_length_list.iter().enumerate() {
                // VOICEVOX ENGINEと挙動を合わせるため、四捨五入ではなく偶数丸めをする
                //
                // https://github.com/VOICEVOX/voicevox_engine/issues/552
                let phoneme_length = ((*phoneme_length * RATE).round_ties_even() / speed_scale)
                    .round_ties_even() as usize;
                let phoneme_id = usize::from(phoneme_data_list[i]);

                for _ in 0..phoneme_length {
                    let mut phonemes_vec = [0.; PhonemeCode::num_phoneme()]; // TODO: Rust 1.89であればサイズが型推論可能になる
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
            const FIX_VOWEL_LENGTH: f32 = 0.15;
            const ADJUST_PITCH: f32 = 0.3;
            const MAX_PITCH: f32 = 6.5;

            let pitch = (last_mora.pitch + ADJUST_PITCH).min(MAX_PITCH);

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
