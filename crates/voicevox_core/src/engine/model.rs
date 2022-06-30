use derive_getters::Getters;
use derive_new::new;

#[allow(dead_code)] // TODO: remove this feature
#[derive(Clone, Debug, new, Getters)]
pub(super) struct MoraModel {
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
pub(super) struct AccentPhraseModel {
    moras: Vec<MoraModel>,
    accent: usize,
    pub(super) pause_mora: Option<MoraModel>,
    pub(super) is_interrogative: bool,
}

#[allow(dead_code)] // TODO: remove this feature
#[derive(new, Getters)]
pub(super) struct AudioQueryModel {
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
