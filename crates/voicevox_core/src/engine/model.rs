use derive_getters::Getters;
use derive_new::new;
use serde::{Deserialize, Serialize};

/* 各フィールドのjsonフィールド名はsnake_caseとする*/

/// モーラ（子音＋母音）ごとの情報。
#[derive(Clone, Debug, new, Getters, Deserialize, Serialize)]
pub struct MoraModel {
    /// 文字。
    text: String,
    /// 子音の音素。
    consonant: Option<String>,
    /// 子音の音長。
    consonant_length: Option<f32>,
    /// 母音の音素。
    vowel: String,
    /// 母音の音長。
    vowel_length: f32,
    /// 音高。
    pitch: f32,
}

/// AccentPhrase (アクセント句ごとの情報)。
#[derive(Clone, Debug, new, Getters, Deserialize, Serialize)]
pub struct AccentPhraseModel {
    /// モーラの配列。
    moras: Vec<MoraModel>,
    /// アクセント箇所。
    accent: usize,
    /// 後ろに無音を付けるかどうか。
    pause_mora: Option<MoraModel>,
    /// 疑問系かどうか。
    #[serde(default)]
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

/// AudioQuery (音声合成用のクエリ)。
#[allow(clippy::too_many_arguments)]
#[derive(Clone, new, Getters, Deserialize, Serialize)]
pub struct AudioQueryModel {
    /// アクセント句の配列。
    accent_phrases: Vec<AccentPhraseModel>,
    /// 全体の話速。
    speed_scale: f32,
    /// 全体の音高。
    pitch_scale: f32,
    /// 全体の抑揚。
    intonation_scale: f32,
    /// 全体の音量。
    volume_scale: f32,
    /// 音声の前の無音時間。
    pre_phoneme_length: f32,
    /// 音声の後の無音時間。
    post_phoneme_length: f32,
    /// 音声データの出力サンプリングレート。
    output_sampling_rate: u32,
    /// 音声データをステレオ出力するか否か。
    output_stereo: bool,
    /// \[読み取り専用\] AquesTalk風記法。
    ///
    /// [`Synthesizer::audio_query`]が返すもののみ`Some`となる。入力としてのAudioQueryでは無視され
    /// る。
    ///
    /// [`Synthesizer::audio_query`]: crate::Synthesizer::audio_query
    kana: Option<String>,
}

impl AudioQueryModel {
    pub(crate) fn with_kana(self, kana: Option<String>) -> Self {
        Self { kana, ..self }
    }
}

#[derive(Deserialize, Serialize)]
pub struct MorphableTargetInfo {
    pub is_morphable: bool,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use serde_json::json;

    use super::AudioQueryModel;

    #[rstest]
    fn check_audio_query_model_json_field_snake_case() {
        let audio_query_model =
            AudioQueryModel::new(vec![], 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, false, None);
        let val = serde_json::to_value(audio_query_model).unwrap();
        check_json_field_snake_case(&val);
    }

    fn check_json_field_snake_case(val: &serde_json::Value) {
        use serde_json::Value::*;

        match val {
            Object(obj) => {
                for (k, v) in obj.iter() {
                    use heck::ToSnakeCase as _;
                    assert_eq!(k.to_snake_case(), *k, "should be snake case {k}");
                    check_json_field_snake_case(v);
                }
            }
            Array(array) => {
                for val in array.iter() {
                    check_json_field_snake_case(val);
                }
            }
            _ => {}
        }
    }

    #[rstest]
    fn it_accepts_json_without_optional_fields() -> anyhow::Result<()> {
        serde_json::from_value::<AudioQueryModel>(json!({
            "accent_phrases": [
                {
                    "moras": [
                        {
                            "text": "ア",
                            "vowel": "a",
                            "vowel_length": 0.0,
                            "pitch": 0.0
                        }
                    ],
                    "accent": 1
                }
            ],
            "speed_scale": 1.0,
            "pitch_scale": 0.0,
            "intonation_scale": 1.0,
            "volume_scale": 1.0,
            "pre_phoneme_length": 0.1,
            "post_phoneme_length": 0.1,
            "output_sampling_rate": 24000,
            "output_stereo": false
        }))?;
        Ok(())
    }
}
