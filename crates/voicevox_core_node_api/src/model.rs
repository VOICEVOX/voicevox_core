use serde::{Deserialize, Serialize};
use voicevox_core::{AccentPhraseModel, AudioQueryModel};

#[derive(Deserialize, Serialize)]
#[napi(object)]
pub struct Mora {
    /// 文字。
    pub text: String,
    /// 子音の音素。
    pub consonant: Option<String>,
    /// 子音の音長。
    #[napi(js_name = "consonant_length")]
    pub consonant_length: Option<f64>,
    /// 母音の音素。
    pub vowel: String,
    /// 母音の音長。
    #[napi(js_name = "vowel_length")]
    pub vowel_length: f64,
    /// 音高。
    pub pitch: f64,
}

#[derive(Deserialize, Serialize)]
#[napi(object)]
pub struct AccentPhrase {
    /// モーラの配列。
    pub moras: Vec<Mora>,
    /// アクセント箇所。
    pub accent: i64,
    /// 後ろに無音を付けるかどうか。
    #[napi(js_name = "pause_mora")]
    pub pause_mora: Option<Mora>,
    /// 疑問系かどうか。
    #[napi(js_name = "is_interrogative")]
    pub is_interrogative: bool,
}

impl AccentPhrase {
    pub(crate) fn convert_from(value: &AccentPhraseModel) -> serde_json::Result<AccentPhrase> {
        serde_json::from_str(&(serde_json::to_string(value)?))
    }

    pub(crate) fn convert_from_slice(
        values: &[AccentPhraseModel],
    ) -> serde_json::Result<Vec<AccentPhrase>> {
        let mut phrases: Vec<AccentPhrase> = Vec::with_capacity(values.len());
        for value in values {
            phrases.push(AccentPhrase::convert_from(&value)?)
        }
        Ok(phrases)
    }

    pub(crate) fn convert(&self) -> serde_json::Result<AccentPhraseModel> {
        serde_json::from_str(&serde_json::to_string(self)?)
    }

    pub(crate) fn convert_slice(
        values: &[AccentPhrase],
    ) -> serde_json::Result<Vec<AccentPhraseModel>> {
        let mut models: Vec<AccentPhraseModel> = Vec::with_capacity(values.len());
        for value in values {
            models.push(value.convert()?)
        }
        Ok(models)
    }
}

#[derive(Deserialize, Serialize)]
#[napi(object)]
pub struct AudioQuery {
    /// アクセント句の配列。
    #[napi(js_name = "accent_phrases")]
    pub accent_phrases: Vec<AccentPhrase>,
    /// 全体の話速。
    #[napi(js_name = "speed_scale")]
    pub speed_scale: f64,
    /// 全体の音高。
    #[napi(js_name = "pitch_scale")]
    pub pitch_scale: f64,
    /// 全体の抑揚。
    #[napi(js_name = "intonation_scale")]
    pub intonation_scale: f64,
    /// 全体の音量。
    #[napi(js_name = "volume_scale")]
    pub volume_scale: f64,
    /// 音声の前の無音時間。
    #[napi(js_name = "pre_phoneme_length")]
    pub pre_phoneme_length: f64,
    /// 音声の後の無音時間。
    #[napi(js_name = "post_phoneme_length")]
    pub post_phoneme_length: f64,
    /// 音声データの出力サンプリングレート。
    #[napi(js_name = "output_sampling_rate")]
    pub output_sampling_rate: u32,
    /// 音声データをステレオ出力するか否か。
    #[napi(js_name = "output_stereo")]
    pub output_stereo: bool,
    /// @readonly AquesTalk風記法。
    ///
    /// {@link blocking.Synthesizer#audio_query}が返すもののみ`string`となる。入力としてのAudioQueryでは無視され
    /// る。
    pub kana: Option<String>,
}

impl AudioQuery {
    pub(crate) fn convert_from(value: &AudioQueryModel) -> serde_json::Result<AudioQuery> {
        serde_json::from_str(&serde_json::to_string(value)?)
    }

    pub(crate) fn convert(&self) -> serde_json::Result<AudioQueryModel> {
        serde_json::from_str(&serde_json::to_string(self)?)
    }
}
