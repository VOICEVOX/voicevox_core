use derive_getters::Getters;
use derive_new::new;
use serde::{Deserialize, Serialize};

/* 各フィールドのjsonフィールド名はcamelCaseとする*/

#[derive(Clone, Debug, new, Getters, Deserialize, Serialize)]
pub struct MoraModel {
    text: String,
    consonant: Option<String>,
    #[serde(rename = "consonantLength")]
    consonant_length: Option<f32>,
    vowel: String,
    #[serde(rename = "vowelLength")]
    vowel_length: f32,
    pitch: f32,
}

#[derive(Clone, Debug, new, Getters, Deserialize, Serialize)]
pub struct AccentPhraseModel {
    moras: Vec<MoraModel>,
    accent: usize,
    #[serde(rename = "pauseMora")]
    pause_mora: Option<MoraModel>,
    #[serde(rename = "isInterrogative")]
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
    #[serde(rename = "accentPhrases")]
    accent_phrases: Vec<AccentPhraseModel>,
    #[serde(rename = "speedScale")]
    speed_scale: f32,
    #[serde(rename = "pitchScale")]
    pitch_scale: f32,
    #[serde(rename = "intonationScale")]
    intonation_scale: f32,
    #[serde(rename = "volumeScale")]
    volume_scale: f32,
    #[serde(rename = "prePhonemeLength")]
    pre_phoneme_length: f32,
    #[serde(rename = "postPhonemeLength")]
    post_phoneme_length: f32,
    #[serde(rename = "outputSamplingRate")]
    output_sampling_rate: u32,
    #[serde(rename = "outputStereo")]
    output_stereo: bool,
    kana: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[rstest]
    fn check_audio_query_model_json_field_camel_case() {
        let audio_query_model =
            AudioQueryModel::new(vec![], 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, false, "".into());
        let val = serde_json::to_value(&audio_query_model).unwrap();
        check_json_field_camel_case(&val);
    }

    fn check_json_field_camel_case(val: &serde_json::Value) {
        use serde_json::Value::*;
        match val {
            Object(obj) => {
                for (k, v) in obj.iter() {
                    assert!(
                        inflections::case::is_camel_case(k),
                        "should be camel case {k}"
                    );
                    check_json_field_camel_case(v);
                }
            }
            Array(array) => {
                for val in array.iter() {
                    check_json_field_camel_case(val);
                }
            }
            _ => {}
        }
    }
}
