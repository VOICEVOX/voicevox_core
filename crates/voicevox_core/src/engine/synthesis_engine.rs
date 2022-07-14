use std::path::Path;

use super::open_jtalk::OpenJtalk;
use super::*;

pub struct SynthesisEngine {
    open_jtalk: OpenJtalk,
    is_openjtalk_dict_loaded: bool,
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
    pub fn new() -> Self {
        Self {
            open_jtalk: OpenJtalk::initialize(),
            is_openjtalk_dict_loaded: false,
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
        let result = self.open_jtalk.load(mecab_dict_dir);
        self.is_openjtalk_dict_loaded = result.is_ok();
        result.map_err(|_| Error::NotLoadedOpenjtalkDict)
    }

    pub fn is_openjtalk_dict_loaded(&self) -> bool {
        self.is_openjtalk_dict_loaded
    }
}
