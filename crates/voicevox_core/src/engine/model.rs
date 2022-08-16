use derive_getters::Getters;
use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, new, Getters, Deserialize, Serialize)]
pub struct MoraModel {
    text: String,
    consonant: Option<String>,
    consonant_length: Option<f32>,
    vowel: String,
    vowel_length: f32,
    pitch: f32,
}

#[derive(Clone, Debug, new, Getters, Deserialize, Serialize)]
pub struct AccentPhraseModel {
    moras: Vec<MoraModel>,
    accent: usize,
    pause_mora: Option<MoraModel>,
    is_interrogative: bool,
}

impl AccentPhraseModel {
    pub(super) fn set_pause_mora(&mut self, pause_mora: Option<MoraModel>) {
        self.pause_mora = pause_mora;
    }

    pub(super) fn set_is_interrogative(&mut self, is_interrogative: bool) {
        self.is_interrogative = is_interrogative;
    }
}

#[allow(clippy::too_many_arguments)]
#[derive(Clone, new, Getters, Deserialize, Serialize)]
pub struct AudioQueryModel {
    accent_phrases: Vec<AccentPhraseModel>,
    speed_scale: f32,
    pitch_scale: f32,
    intonation_scale: f32,
    volume_scale: f32,
    pre_phoneme_length: f32,
    post_phoneme_length: f32,
    output_sampling_rate: u32,
    output_stereo: bool,
    #[allow(dead_code)]
    kana: String,
}
