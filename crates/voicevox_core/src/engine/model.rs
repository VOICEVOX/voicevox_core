use derive_getters::Getters;
use derive_new::new;

#[allow(dead_code)] // TODO: remove this feature
#[derive(Clone, Debug, new, Getters)]
pub struct MoraModel {
    text: String,
    consonant: Option<String>,
    consonant_length: Option<f32>,
    vowel: String,
    #[new(default)]
    vowel_length: f32,
    #[new(default)]
    pitch: f32,
}

#[allow(dead_code)] // TODO: remove this feature
#[derive(Debug, new, Getters)]
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

#[allow(dead_code, clippy::too_many_arguments)] // TODO: remove allow(dead_code)
#[derive(new, Getters)]
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
    kana: String,
}
