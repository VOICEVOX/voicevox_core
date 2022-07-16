use std::path::Path;

use super::full_context_label::Utterance;
use super::internal::InferenceCore;
use super::open_jtalk::OpenJtalk;
use super::*;

const UNVOICED_MORA_PHONEME_LIST: &[&str] = &["A", "I", "U", "E", "O", "cl", "pau"];

const MORA_PHONEME_LIST: &[&str] = &[
    "a", "i", "u", "e", "o", "N", "A", "I", "U", "E", "O", "cl", "pau",
];

/*
 * TODO: OpenJtalk機能を使用するようになったら、allow(dead_code),allow(unused_variables)を消す
 */
#[allow(dead_code)]
pub struct SynthesisEngine {
    open_jtalk: OpenJtalk,
    inference_core: InferenceCore,
}

#[allow(unsafe_code)]
unsafe impl Send for SynthesisEngine {}
#[allow(unsafe_code)]
unsafe impl Sync for SynthesisEngine {}

#[allow(dead_code)]
#[allow(unused_variables)]
impl SynthesisEngine {
    const DEFAULT_SAMPLING_RATE: usize = 24000;

    #[allow(clippy::new_without_default)]
    pub fn new(inference_core: InferenceCore) -> Self {
        Self {
            open_jtalk: OpenJtalk::initialize(),
            inference_core,
        }
    }

    pub fn inference_core(&self) -> &InferenceCore {
        &self.inference_core
    }

    pub fn inference_core_mut(&mut self) -> &mut InferenceCore {
        &mut self.inference_core
    }

    pub fn create_accent_phrases(
        &mut self,
        text: impl AsRef<str>,
        speaker_id: usize,
    ) -> Result<Vec<AccentPhraseModel>> {
        if text.as_ref().is_empty() {
            return Ok(Vec::new());
        }

        let utterance = Utterance::extract_full_context_label(&mut self.open_jtalk, text.as_ref())
            .map_err(Error::FailedExtractFullContextLabel)?;

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
                                let mut mora_text = mora
                                    .phonemes()
                                    .iter()
                                    .map(|phoneme| phoneme.phoneme().to_string())
                                    .collect::<Vec<_>>()
                                    .join("");
                                mora_text = mora_text.to_lowercase();
                                if mora_text == "n" {
                                    mora_text = String::from("N");
                                }

                                let (consonant, consonant_length) =
                                    if let Some(consonant) = mora.consonant() {
                                        (Some(consonant.phoneme().to_string()), Some(0.))
                                    } else {
                                        (None, None)
                                    };

                                MoraModel::new(
                                    mora_text,
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
        speaker_id: usize,
    ) -> Result<Vec<AccentPhraseModel>> {
        let accent_phrases = self.replace_phoneme_length(accent_phrases, speaker_id)?;
        self.replace_mora_pitch(&accent_phrases, speaker_id)
    }

    pub fn replace_phoneme_length(
        &mut self,
        accent_phrases: &[AccentPhraseModel],
        speaker_id: usize,
    ) -> Result<Vec<AccentPhraseModel>> {
        let (_, phoneme_data_list) = SynthesisEngine::initial_process(accent_phrases);

        let (_, _, vowel_indexes_data) = split_mora(&phoneme_data_list);

        let phoneme_list_s: Vec<i64> = phoneme_data_list
            .iter()
            .map(|phoneme_data| phoneme_data.phoneme_id())
            .collect();
        let phoneme_length = self
            .inference_core_mut()
            .yukarin_s_forward(&phoneme_list_s, speaker_id)?;

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
                                mora.consonant().as_ref().map(|s| {
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
                            Some(phoneme_length[vowel_indexes_data[index + 1] as usize]),
                            pause_mora.vowel().clone(),
                            *pause_mora.vowel_length(),
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
        speaker_id: usize,
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

        let mut f0_list = self.inference_core_mut().yukarin_sa_forward(
            vowel_phoneme_list.len() as i64,
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
        &self,
        query: &AudioQueryModel,
        speaker_id: usize,
        enable_interrogative_upspeak: bool,
    ) -> Result<Vec<f32>> {
        unimplemented!()
    }

    pub fn synthesis_wave_format(
        &self,
        query: &AudioQueryModel,
        speaker_id: usize,
        binary_size: usize,
        enable_interrogative_upspeak: bool,
    ) -> Result<Vec<u8>> {
        unimplemented!()
    }

    pub fn load_openjtalk_dict(&mut self, mecab_dict_dir: impl AsRef<Path>) -> Result<()> {
        unimplemented!()
    }

    pub fn is_openjtalk_dict_loaded(&self) -> bool {
        unimplemented!()
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
