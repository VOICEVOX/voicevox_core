#[allow(dead_code)] // TODO: remove this feature
#[derive(Clone)]
pub(super) struct MoraModel {
    pub text: String,
    pub consonant: Option<String>,
    pub consonant_length: Option<f32>,
    pub vowel: String,
    pub vowel_length: f32,
    pub pitch: f32,
}

#[allow(dead_code)] // TODO: remove this feature
pub(super) struct AccentPhraseModel {
    pub moras: Vec<MoraModel>,
    pub accent: usize,
    pub pause_mora: Option<MoraModel>,
    pub is_interrogative: bool,
}

#[allow(dead_code)] // TODO: remove this feature
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
