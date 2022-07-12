use std::path::Path;

use super::open_jtalk::OpenJtalk;
use super::*;

/*
 * TODO: OpenJtalk機能を使用するようになったら、allow(dead_code),allow(unused_variables)を消す
 */
#[allow(dead_code)]
pub struct SynthesisEngine {
    open_jtalk: OpenJtalk,
}

#[allow(dead_code)]
#[allow(unused_variables)]
impl SynthesisEngine {
    const DEFAULT_SAMPLING_RATE: usize = 24000;

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            open_jtalk: OpenJtalk::initialize(),
        }
    }

    pub fn create_accent_phrases(
        &self,
        text: impl AsRef<str>,
        speaker_id: usize,
    ) -> Result<Vec<AccentPhraseModel>> {
        unimplemented!()
    }

    pub fn replace_mora_data(
        &self,
        accent_phrases: &[AccentPhraseModel],
        speaker_id: usize,
    ) -> Result<Vec<AccentPhraseModel>> {
        unimplemented!()
    }

    pub fn replace_phoneme_length(
        &self,
        accent_phrases: &[AccentPhraseModel],
        speaker_id: usize,
    ) -> Result<Vec<AccentPhraseModel>> {
        unimplemented!()
    }

    pub fn replace_mora_pitch(
        &self,
        accent_phrases: &[AccentPhraseModel],
        speaker_id: usize,
    ) -> Result<Vec<AccentPhraseModel>> {
        unimplemented!()
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
}
