use derive_new::new;
use std::io::{Cursor, Write};
use std::path::Path;

use super::full_context_label::Utterance;
use super::open_jtalk::OpenJtalk;
use super::*;
use crate::InferenceCore;

const UNVOICED_MORA_PHONEME_LIST: &[&str] = &["A", "I", "U", "E", "O", "cl", "pau"];

const MORA_PHONEME_LIST: &[&str] = &[
    "a", "i", "u", "e", "o", "N", "A", "I", "U", "E", "O", "cl", "pau",
];

#[derive(new)]
pub struct SynthesisEngine {
    inference_core: InferenceCore,
    open_jtalk: OpenJtalk,
}

#[allow(unsafe_code)]
unsafe impl Send for SynthesisEngine {}
#[allow(unsafe_code)]
unsafe impl Sync for SynthesisEngine {}

impl SynthesisEngine {
    pub const DEFAULT_SAMPLING_RATE: u32 = 24000;

    pub fn inference_core(&self) -> &InferenceCore {
        &self.inference_core
    }

    pub fn inference_core_mut(&mut self) -> &mut InferenceCore {
        &mut self.inference_core
    }

    pub fn create_accent_phrases(
        &mut self,
        text: impl AsRef<str>,
        speaker_id: u32,
    ) -> Result<Vec<AccentPhraseModel>> {
        if text.as_ref().is_empty() {
            return Ok(Vec::new());
        }

        let utterance = Utterance::extract_full_context_label(&mut self.open_jtalk, text.as_ref())?;

        let accent_phrases: Vec<AccentPhraseModel> = utterance
            .breath_groups()
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut accum_vec, (i, breath_group)| {
                accum_vec.extend(breath_group.accent_phrases().iter().enumerate().map(
                    |(j, accent_phrase)| {
                        let moras = accent_phrase
                            .moras()
                            .iter()
                            .map(|mora| {
                                let mora_text = mora
                                    .phonemes()
                                    .iter()
                                    .map(|phoneme| phoneme.phoneme().to_string())
                                    .collect::<Vec<_>>()
                                    .join("");

                                let (consonant, consonant_length) =
                                    if let Some(consonant) = mora.consonant() {
                                        (Some(consonant.phoneme().to_string()), Some(0.))
                                    } else {
                                        (None, None)
                                    };

                                MoraModel::new(
                                    mora_to_text(&mora_text),
                                    consonant,
                                    consonant_length,
                                    mora.vowel().phoneme().into(),
                                    0.,
                                    0.,
                                )
                            })
                            .collect();

                        let pause_mora = if i != utterance.breath_groups().len() - 1
                            && j == breath_group.accent_phrases().len() - 1
                        {
                            Some(MoraModel::new(
                                "、".into(),
                                None,
                                None,
                                "pau".into(),
                                0.,
                                0.,
                            ))
                        } else {
                            None
                        };

                        AccentPhraseModel::new(
                            moras,
                            *accent_phrase.accent(),
                            pause_mora,
                            *accent_phrase.is_interrogative(),
                        )
                    },
                ));

                accum_vec
            });

        self.replace_mora_data(&accent_phrases, speaker_id)
    }

    pub fn replace_mora_data(
        &mut self,
        accent_phrases: &[AccentPhraseModel],
        speaker_id: u32,
    ) -> Result<Vec<AccentPhraseModel>> {
        let accent_phrases = self.replace_phoneme_length(accent_phrases, speaker_id)?;
        self.replace_mora_pitch(&accent_phrases, speaker_id)
    }

    pub fn replace_phoneme_length(
        &mut self,
        accent_phrases: &[AccentPhraseModel],
        speaker_id: u32,
    ) -> Result<Vec<AccentPhraseModel>> {
        let (_, phoneme_data_list) = SynthesisEngine::initial_process(accent_phrases);

        let (_, _, vowel_indexes_data) = split_mora(&phoneme_data_list);

        let phoneme_list_s: Vec<i64> = phoneme_data_list
            .iter()
            .map(|phoneme_data| phoneme_data.phoneme_id())
            .collect();
        let phoneme_length = self
            .inference_core_mut()
            .predict_duration(&phoneme_list_s, speaker_id)?;

        let mut index = 0;
        let new_accent_phrases = accent_phrases
            .iter()
            .map(|accent_phrase| {
                AccentPhraseModel::new(
                    accent_phrase
                        .moras()
                        .iter()
                        .map(|mora| {
                            let new_mora = MoraModel::new(
                                mora.text().clone(),
                                mora.consonant().clone(),
                                mora.consonant().as_ref().map(|_| {
                                    phoneme_length[vowel_indexes_data[index + 1] as usize - 1]
                                }),
                                mora.vowel().clone(),
                                phoneme_length[vowel_indexes_data[index + 1] as usize],
                                *mora.pitch(),
                            );
                            index += 1;
                            new_mora
                        })
                        .collect(),
                    *accent_phrase.accent(),
                    accent_phrase.pause_mora().as_ref().map(|pause_mora| {
                        let new_pause_mora = MoraModel::new(
                            pause_mora.text().clone(),
                            pause_mora.consonant().clone(),
                            *pause_mora.consonant_length(),
                            pause_mora.vowel().clone(),
                            phoneme_length[vowel_indexes_data[index + 1] as usize],
                            *pause_mora.pitch(),
                        );
                        index += 1;
                        new_pause_mora
                    }),
                    *accent_phrase.is_interrogative(),
                )
            })
            .collect();

        Ok(new_accent_phrases)
    }

    pub fn replace_mora_pitch(
        &mut self,
        accent_phrases: &[AccentPhraseModel],
        speaker_id: u32,
    ) -> Result<Vec<AccentPhraseModel>> {
        let (_, phoneme_data_list) = SynthesisEngine::initial_process(accent_phrases);

        let mut base_start_accent_list = vec![0];
        let mut base_end_accent_list = vec![0];
        let mut base_start_accent_phrase_list = vec![0];
        let mut base_end_accent_phrase_list = vec![0];
        for accent_phrase in accent_phrases {
            let mut accent: usize = if *accent_phrase.accent() == 1 { 0 } else { 1 };
            SynthesisEngine::create_one_accent_list(
                &mut base_start_accent_list,
                accent_phrase,
                accent as i32,
            );

            accent = *accent_phrase.accent() - 1;
            SynthesisEngine::create_one_accent_list(
                &mut base_end_accent_list,
                accent_phrase,
                accent as i32,
            );
            SynthesisEngine::create_one_accent_list(
                &mut base_start_accent_phrase_list,
                accent_phrase,
                0,
            );
            SynthesisEngine::create_one_accent_list(
                &mut base_end_accent_phrase_list,
                accent_phrase,
                -1,
            );
        }
        base_start_accent_list.push(0);
        base_end_accent_list.push(0);
        base_start_accent_phrase_list.push(0);
        base_end_accent_phrase_list.push(0);

        let (consonant_phoneme_data_list, vowel_phoneme_data_list, vowel_indexes) =
            split_mora(&phoneme_data_list);

        let consonant_phoneme_list: Vec<i64> = consonant_phoneme_data_list
            .iter()
            .map(|phoneme_data| phoneme_data.phoneme_id())
            .collect();
        let vowel_phoneme_list: Vec<i64> = vowel_phoneme_data_list
            .iter()
            .map(|phoneme_data| phoneme_data.phoneme_id())
            .collect();

        let mut start_accent_list = Vec::with_capacity(vowel_indexes.len());
        let mut end_accent_list = Vec::with_capacity(vowel_indexes.len());
        let mut start_accent_phrase_list = Vec::with_capacity(vowel_indexes.len());
        let mut end_accent_phrase_list = Vec::with_capacity(vowel_indexes.len());

        for vowel_index in vowel_indexes {
            start_accent_list.push(base_start_accent_list[vowel_index as usize]);
            end_accent_list.push(base_end_accent_list[vowel_index as usize]);
            start_accent_phrase_list.push(base_start_accent_phrase_list[vowel_index as usize]);
            end_accent_phrase_list.push(base_end_accent_phrase_list[vowel_index as usize]);
        }

        let mut f0_list = self.inference_core_mut().predict_intonation(
            vowel_phoneme_list.len(),
            &vowel_phoneme_list,
            &consonant_phoneme_list,
            &start_accent_list,
            &end_accent_list,
            &start_accent_phrase_list,
            &end_accent_phrase_list,
            speaker_id,
        )?;

        for i in 0..vowel_phoneme_data_list.len() {
            if UNVOICED_MORA_PHONEME_LIST
                .iter()
                .any(|phoneme| *phoneme == vowel_phoneme_data_list[i].phoneme())
            {
                f0_list[i] = 0.;
            }
        }

        let mut index = 0;
        let new_accent_phrases = accent_phrases
            .iter()
            .map(|accent_phrase| {
                AccentPhraseModel::new(
                    accent_phrase
                        .moras()
                        .iter()
                        .map(|mora| {
                            let new_mora = MoraModel::new(
                                mora.text().clone(),
                                mora.consonant().clone(),
                                *mora.consonant_length(),
                                mora.vowel().clone(),
                                *mora.vowel_length(),
                                f0_list[index + 1],
                            );
                            index += 1;
                            new_mora
                        })
                        .collect(),
                    *accent_phrase.accent(),
                    accent_phrase.pause_mora().as_ref().map(|pause_mora| {
                        let new_pause_mora = MoraModel::new(
                            pause_mora.text().clone(),
                            pause_mora.consonant().clone(),
                            *pause_mora.consonant_length(),
                            pause_mora.vowel().clone(),
                            *pause_mora.vowel_length(),
                            f0_list[index + 1],
                        );
                        index += 1;
                        new_pause_mora
                    }),
                    *accent_phrase.is_interrogative(),
                )
            })
            .collect();

        Ok(new_accent_phrases)
    }

    pub fn synthesis(
        &mut self,
        query: &AudioQueryModel,
        speaker_id: u32,
        enable_interrogative_upspeak: bool,
    ) -> Result<Vec<f32>> {
        let speed_scale = *query.speed_scale();
        let pitch_scale = *query.pitch_scale();
        let intonation_scale = *query.intonation_scale();
        let pre_phoneme_length = *query.pre_phoneme_length();
        let post_phoneme_length = *query.post_phoneme_length();

        let accent_phrases = if enable_interrogative_upspeak {
            adjust_interrogative_accent_phrases(query.accent_phrases().as_slice())
        } else {
            query.accent_phrases().clone()
        };

        let (flatten_moras, phoneme_data_list) = SynthesisEngine::initial_process(&accent_phrases);

        let mut phoneme_length_list = vec![pre_phoneme_length];
        let mut f0_list = vec![0.];
        let mut voiced_list = vec![false];
        {
            let mut sum_of_f0_bigger_than_zero = 0.;
            let mut count_of_f0_bigger_than_zero = 0;

            for mora in flatten_moras {
                let consonant_length = *mora.consonant_length();
                let vowel_length = *mora.vowel_length();
                let pitch = *mora.pitch();

                if let Some(consonant_length) = consonant_length {
                    phoneme_length_list.push(consonant_length);
                }
                phoneme_length_list.push(vowel_length);

                let f0_single = pitch * 2.0_f32.powf(pitch_scale);
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

        let mut phoneme: Vec<Vec<f32>> = Vec::new();
        let mut f0: Vec<f32> = Vec::new();
        {
            const RATE: f32 = 24000. / 256.;
            let mut sum_of_phoneme_length = 0;
            let mut count_of_f0 = 0;
            let mut vowel_indexes_index = 0;

            for (i, phoneme_length) in phoneme_length_list.iter().enumerate() {
                let phoneme_length =
                    ((*phoneme_length * RATE).round() / speed_scale).round() as usize;
                let phoneme_id = phoneme_data_list[i].phoneme_id();

                for _ in 0..phoneme_length {
                    let mut phonemes_vec = vec![0.; OjtPhoneme::num_phoneme()];
                    phonemes_vec[phoneme_id as usize] = 1.;
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

        // 2次元のvectorを1次元に変換し、アドレスを連続させる
        let flatten_phoneme = phoneme.into_iter().flatten().collect::<Vec<_>>();

        self.inference_core_mut().decode(
            f0.len(),
            OjtPhoneme::num_phoneme(),
            &f0,
            &flatten_phoneme,
            speaker_id,
        )
    }

    pub fn synthesis_wave_format(
        &mut self,
        query: &AudioQueryModel,
        speaker_id: u32,
        enable_interrogative_upspeak: bool,
    ) -> Result<Vec<u8>> {
        let wave = self.synthesis(query, speaker_id, enable_interrogative_upspeak)?;

        let volume_scale = *query.volume_scale();
        let output_stereo = *query.output_stereo();
        // TODO: 44.1kHzなどの対応
        let output_sampling_rate = *query.output_sampling_rate();

        let num_channels: u16 = if output_stereo { 2 } else { 1 };
        let bit_depth: u16 = 16;
        let repeat_count: u32 =
            (output_sampling_rate / Self::DEFAULT_SAMPLING_RATE) * num_channels as u32;
        let block_size: u16 = bit_depth * num_channels / 8;

        let bytes_size = wave.len() as u32 * repeat_count * 2;
        let wave_size = bytes_size + 44;

        let buf: Vec<u8> = Vec::with_capacity(wave_size as usize);
        let mut cur = Cursor::new(buf);

        cur.write_all("RIFF".as_bytes()).unwrap();
        cur.write_all(&(wave_size - 8).to_le_bytes()).unwrap();
        cur.write_all("WAVEfmt ".as_bytes()).unwrap();
        cur.write_all(&16_u32.to_le_bytes()).unwrap(); // fmt header length
        cur.write_all(&1_u16.to_le_bytes()).unwrap(); //linear PCM
        cur.write_all(&num_channels.to_le_bytes()).unwrap();
        cur.write_all(&output_sampling_rate.to_le_bytes()).unwrap();

        let block_rate = output_sampling_rate * block_size as u32;

        cur.write_all(&block_rate.to_le_bytes()).unwrap();
        cur.write_all(&block_size.to_le_bytes()).unwrap();
        cur.write_all(&bit_depth.to_le_bytes()).unwrap();
        cur.write_all("data".as_bytes()).unwrap();
        cur.write_all(&bytes_size.to_le_bytes()).unwrap();

        for value in wave {
            // clip
            let v = (value * volume_scale).max(-1.).min(1.);
            let data = (v * 0x7fff as f32) as i16;
            for _ in 0..repeat_count {
                cur.write_all(&data.to_le_bytes()).unwrap();
            }
        }

        Ok(cur.into_inner())
    }

    pub fn load_openjtalk_dict(&mut self, mecab_dict_dir: impl AsRef<Path>) -> Result<()> {
        self.open_jtalk
            .load(mecab_dict_dir)
            .map_err(|_| Error::NotLoadedOpenjtalkDict)
    }

    pub fn is_openjtalk_dict_loaded(&self) -> bool {
        self.open_jtalk.dict_loaded()
    }

    fn initial_process(accent_phrases: &[AccentPhraseModel]) -> (Vec<MoraModel>, Vec<OjtPhoneme>) {
        let flatten_moras = to_flatten_moras(accent_phrases);

        let mut phoneme_strings = vec!["pau".to_string()];
        for mora in flatten_moras.iter() {
            if let Some(consonant) = mora.consonant() {
                phoneme_strings.push(consonant.clone())
            }
            phoneme_strings.push(mora.vowel().clone());
        }
        phoneme_strings.push("pau".to_string());

        let phoneme_data_list = to_phoneme_data_list(&phoneme_strings);

        (flatten_moras, phoneme_data_list)
    }

    fn create_one_accent_list(
        accent_list: &mut Vec<i64>,
        accent_phrase: &AccentPhraseModel,
        point: i32,
    ) {
        let mut one_accent_list: Vec<i64> = Vec::new();

        for (i, mora) in accent_phrase.moras().iter().enumerate() {
            let value = if i as i32 == point
                || (point < 0 && i == (accent_phrase.moras().len() as i32 + point) as usize)
            {
                1
            } else {
                0
            };
            one_accent_list.push(value as i64);
            if mora.consonant().is_some() {
                one_accent_list.push(value as i64);
            }
        }
        if accent_phrase.pause_mora().is_some() {
            one_accent_list.push(0);
        }
        accent_list.extend(one_accent_list)
    }
}

pub fn to_flatten_moras(accent_phrases: &[AccentPhraseModel]) -> Vec<MoraModel> {
    let mut flatten_moras = Vec::new();

    for accent_phrase in accent_phrases {
        let moras = accent_phrase.moras();
        for mora in moras {
            flatten_moras.push(mora.clone());
        }
        if let Some(pause_mora) = accent_phrase.pause_mora() {
            flatten_moras.push(pause_mora.clone());
        }
    }

    flatten_moras
}

pub fn to_phoneme_data_list<T: AsRef<str>>(phoneme_str_list: &[T]) -> Vec<OjtPhoneme> {
    OjtPhoneme::convert(
        phoneme_str_list
            .iter()
            .enumerate()
            .map(|(i, s)| OjtPhoneme::new(s.as_ref().to_string(), i as f32, i as f32 + 1.))
            .collect::<Vec<OjtPhoneme>>()
            .as_slice(),
    )
}

pub fn split_mora(phoneme_list: &[OjtPhoneme]) -> (Vec<OjtPhoneme>, Vec<OjtPhoneme>, Vec<i64>) {
    let mut vowel_indexes = Vec::new();
    for (i, phoneme) in phoneme_list.iter().enumerate() {
        if MORA_PHONEME_LIST
            .iter()
            .any(|mora_phoneme| *mora_phoneme == phoneme.phoneme())
        {
            vowel_indexes.push(i as i64);
        }
    }

    let vowel_phoneme_list = vowel_indexes
        .iter()
        .map(|vowel_index| phoneme_list[*vowel_index as usize].clone())
        .collect();

    let mut consonant_phoneme_list = vec![OjtPhoneme::default()];
    for i in 0..(vowel_indexes.len() - 1) {
        let prev = vowel_indexes[i];
        let next = vowel_indexes[i + 1];
        if next - prev == 1 {
            consonant_phoneme_list.push(OjtPhoneme::default());
        } else {
            consonant_phoneme_list.push(phoneme_list[next as usize - 1].clone());
        }
    }

    (consonant_phoneme_list, vowel_phoneme_list, vowel_indexes)
}

fn mora_to_text(mora: impl AsRef<str>) -> String {
    let last_char = mora.as_ref().chars().last().unwrap();
    let mora = if ['A', 'I', 'U', 'E', 'O'].contains(&last_char) {
        format!(
            "{}{}",
            &mora.as_ref()[0..mora.as_ref().len() - 1],
            last_char.to_lowercase()
        )
    } else {
        mora.as_ref().to_string()
    };
    // もしカタカナに変換できなければ、引数で与えた文字列がそのまま返ってくる
    mora_list::mora2text(&mora).to_string()
}

fn adjust_interrogative_accent_phrases(
    accent_phrases: &[AccentPhraseModel],
) -> Vec<AccentPhraseModel> {
    accent_phrases
        .iter()
        .map(|accent_phrase| {
            AccentPhraseModel::new(
                adjust_interrogative_moras(accent_phrase),
                *accent_phrase.accent(),
                accent_phrase.pause_mora().clone(),
                *accent_phrase.is_interrogative(),
            )
        })
        .collect()
}

fn adjust_interrogative_moras(accent_phrase: &AccentPhraseModel) -> Vec<MoraModel> {
    let moras = accent_phrase.moras();
    if *accent_phrase.is_interrogative() && !moras.is_empty() {
        let last_mora = moras.last().unwrap();
        let last_mora_pitch = *last_mora.pitch();
        if last_mora_pitch != 0.0 {
            let mut new_moras: Vec<MoraModel> = Vec::with_capacity(moras.len() + 1);
            new_moras.extend_from_slice(moras.as_slice());
            let interrogative_mora = make_interrogative_mora(last_mora);
            new_moras.push(interrogative_mora);
            return new_moras;
        }
    }
    moras.clone()
}

fn make_interrogative_mora(last_mora: &MoraModel) -> MoraModel {
    const FIX_VOWEL_LENGTH: f32 = 0.15;
    const ADJUST_PITCH: f32 = 0.3;
    const MAX_PITCH: f32 = 6.5;

    let pitch = (*last_mora.pitch() + ADJUST_PITCH).min(MAX_PITCH);

    MoraModel::new(
        mora_to_text(last_mora.vowel()),
        None,
        None,
        last_mora.vowel().clone(),
        FIX_VOWEL_LENGTH,
        pitch,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use crate::*;

    #[rstest]
    #[async_std::test]
    async fn load_openjtalk_dict_works() {
        let core = InferenceCore::new(false, None);
        let mut synthesis_engine = SynthesisEngine::new(core, OpenJtalk::initialize());
        let open_jtalk_dic_dir = download_open_jtalk_dict_if_no_exists().await;

        let result = synthesis_engine.load_openjtalk_dict(&open_jtalk_dic_dir);
        assert_eq!(result, Ok(()));

        let result = synthesis_engine.load_openjtalk_dict("");
        assert_eq!(result, Err(Error::NotLoadedOpenjtalkDict));
    }

    #[rstest]
    #[async_std::test]
    async fn is_openjtalk_dict_loaded_works() {
        let core = InferenceCore::new(false, None);
        let mut synthesis_engine = SynthesisEngine::new(core, OpenJtalk::initialize());
        let open_jtalk_dic_dir = download_open_jtalk_dict_if_no_exists().await;

        let _ = synthesis_engine.load_openjtalk_dict(&open_jtalk_dic_dir);
        assert_eq!(synthesis_engine.is_openjtalk_dict_loaded(), true);

        let _ = synthesis_engine.load_openjtalk_dict("");
        assert_eq!(synthesis_engine.is_openjtalk_dict_loaded(), false);
    }

    #[rstest]
    #[async_std::test]
    async fn create_accent_phrases_works() {
        let mut core = InferenceCore::new(true, None);
        core.initialize(false, 0, true).unwrap();
        let mut synthesis_engine = SynthesisEngine::new(core, OpenJtalk::initialize());
        let open_jtalk_dic_dir = download_open_jtalk_dict_if_no_exists().await;

        let _ = synthesis_engine.load_openjtalk_dict(&open_jtalk_dic_dir);
        let accent_phrases = synthesis_engine
            .create_accent_phrases("同じ、文章、です。完全に、同一です。", 0)
            .unwrap();
        assert_eq!(accent_phrases.len(), 5);

        // 入力テキストに「、」や「。」などの句読点が含まれていたときに
        // AccentPhraseModel の pause_mora に期待する値をテスト

        assert!(
            accent_phrases[0].pause_mora().is_some(),
            "accent_phrases[0].pause_mora() is None"
        );
        assert!(
            accent_phrases[1].pause_mora().is_some(),
            "accent_phrases[1].pause_mora() is None"
        );
        assert!(
            accent_phrases[2].pause_mora().is_some(),
            "accent_phrases[2].pause_mora() is None"
        );
        assert!(
            accent_phrases[3].pause_mora().is_some(),
            "accent_phrases[3].pause_mora() is None"
        );
        assert!(
            accent_phrases[4].pause_mora().is_none(), // 文末の句読点は削除される
            "accent_phrases[4].pause_mora() is not None"
        );

        for accent_phrase in accent_phrases.iter().take(4) {
            let pause_mora = accent_phrase.pause_mora().clone().unwrap();
            assert_eq!(pause_mora.text(), "、");
            assert_eq!(pause_mora.consonant(), &None);
            assert_eq!(pause_mora.consonant_length(), &None);
            assert_eq!(pause_mora.vowel(), "pau");
            assert_ne!(
                pause_mora.vowel_length(),
                &0.0,
                "pause_mora.vowel_length() should not be 0.0"
            );
        }
    }
}
