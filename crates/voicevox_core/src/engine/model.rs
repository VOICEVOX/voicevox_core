use serde::{Deserialize, Serialize};

/* 各フィールドのjsonフィールド名はsnake_caseとする*/

/// モーラ（子音＋母音）ごとの情報。
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct Mora {
    /// 文字。
    pub text: String,
    /// 子音の音素。
    pub consonant: Option<String>,
    /// 子音の音長。
    pub consonant_length: Option<f32>,
    /// 母音の音素。
    pub vowel: String,
    /// 母音の音長。
    pub vowel_length: f32,
    /// 音高。
    pub pitch: f32,
}

/// AccentPhrase (アクセント句ごとの情報)。
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct AccentPhrase {
    /// モーラの配列。
    pub moras: Vec<Mora>,
    /// アクセント箇所。
    pub accent: usize,
    /// 後ろに無音を付けるかどうか。
    pub pause_mora: Option<Mora>,
    /// 疑問系かどうか。
    #[serde(default)]
    pub is_interrogative: bool,
}

impl AccentPhrase {
    pub(super) fn set_pause_mora(&mut self, pause_mora: Option<Mora>) {
        self.pause_mora = pause_mora;
    }

    pub(super) fn set_is_interrogative(&mut self, is_interrogative: bool) {
        self.is_interrogative = is_interrogative;
    }
}

/// AudioQuery (音声合成用のクエリ)。
///
/// # Serialization
///
/// VOICEVOX ENGINEと同じスキーマになっている。ただし今後の破壊的変更にて変わる可能性がある。[データのシリアライゼーション]を参照。
///
/// [データのシリアライゼーション]: https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md
#[derive(Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AudioQuery {
    /// アクセント句の配列。
    pub accent_phrases: Vec<AccentPhrase>,
    /// 全体の話速。
    #[serde(rename = "speedScale")]
    pub speed_scale: f32,
    /// 全体の音高。
    #[serde(rename = "pitchScale")]
    pub pitch_scale: f32,
    /// 全体の抑揚。
    #[serde(rename = "intonationScale")]
    pub intonation_scale: f32,
    /// 全体の音量。
    #[serde(rename = "volumeScale")]
    pub volume_scale: f32,
    /// 音声の前の無音時間。
    #[serde(rename = "prePhonemeLength")]
    pub pre_phoneme_length: f32,
    /// 音声の後の無音時間。
    #[serde(rename = "postPhonemeLength")]
    pub post_phoneme_length: f32,
    /// 音声データの出力サンプリングレート。
    #[serde(rename = "outputSamplingRate")]
    pub output_sampling_rate: u32,
    /// 音声データをステレオ出力するか否か。
    #[serde(rename = "outputStereo")]
    pub output_stereo: bool,
    /// \[読み取り専用\] AquesTalk風記法。
    ///
    /// [`Synthesizer::create_audio_query`]が返すもののみ`Some`となる。入力としてのAudioQueryでは無視され
    /// る。
    ///
    /// [`Synthesizer::create_audio_query`]: crate::blocking::Synthesizer::create_audio_query
    pub kana: Option<String>,
}

impl AudioQuery {
    pub(crate) fn with_kana(self, kana: Option<String>) -> Self {
        Self { kana, ..self }
    }
}

/// 音符のID。
#[derive(Clone, Deserialize, Serialize)]
pub struct NoteId(pub String);

/// 音符ごとの情報。
pub struct Note {
    /// ID。
    pub id: Option<NoteId>,

    /// 音階。
    pub key: Option<usize>,

    /// 音符のフレーム長。
    pub frame_length: usize,

    /// 音符の歌詞。
    pub lyric: String,
}

/// 楽譜情報。
pub struct Score {
    /// 音符のリスト。
    pub notes: Vec<Note>,
}

/// 音素の情報。
#[derive(Deserialize, Serialize)]
pub struct FramePhoneme {
    /// 音素。
    pub phoneme: String,

    /// 音素のフレーム長。
    pub frame_length: usize,

    /// 音符のID。
    pub note_id: Option<NoteId>,
}

/// フレームごとの音声合成用のクエリ。
///
/// # Serialization
///
/// VOICEVOX ENGINEと同じスキーマになっている。ただし今後の破壊的変更にて変わる可能性がある。[データのシリアライゼーション]を参照。
///
/// [データのシリアライゼーション]: https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameAudioQuery {
    /// フレームごとの基本周波数。
    pub f0: Vec<f32>,

    /// フレームごとの音量。
    pub volume: Vec<f32>,

    /// 音素のリスト。
    pub phonemes: Vec<FramePhoneme>,

    /// 全体の音量。
    pub volume_scale: f32,

    /// 音声データの出力サンプリングレート。
    pub output_sample_rate: f32,

    /// 音声データをステレオ出力するか否か。
    pub output_stereo: bool,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::AudioQuery;

    #[rstest]
    fn it_accepts_json_without_optional_fields() -> anyhow::Result<()> {
        serde_json::from_value::<AudioQuery>(json!({
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
            "speedScale": 1.0,
            "pitchScale": 0.0,
            "intonationScale": 1.0,
            "volumeScale": 1.0,
            "prePhonemeLength": 0.1,
            "postPhonemeLength": 0.1,
            "outputSamplingRate": 24000,
            "outputStereo": false
        }))?;
        Ok(())
    }
}
