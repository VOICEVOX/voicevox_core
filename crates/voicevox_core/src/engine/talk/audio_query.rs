mod validated;

use serde::{Deserialize, Serialize};

pub(crate) use self::validated::{
    LengthedPhoneme, ValidatedAccentPhrase, ValidatedAudioQuery, ValidatedMora,
};

/* 各フィールドのjsonフィールド名はsnake_caseとする*/

/// モーラ（子音＋母音）ごとの情報。
///
/// # Validation
///
/// この構造体は不正な状態を表現しうる。どのような状態が不正なのかについては[`validate`メソッド]を参照。この構造体を使う関数は、不正な状態に対して[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
///
/// [`Deserialize`]時には、不正な状態であるかの検証は行われない。外部からのデータが不正でないことを確かめるには、デシリアライズ後に`validate`メソッドを用いる必要がある。
///
/// ```
/// # use voicevox_core::Mora;
/// # let json = r#"{ "text": "ア", "vowel": "a", "vowel_length": 0, "pitch": 0.0 }"#;
/// let mora = serde_json::from_str::<Mora>(json)?;
/// mora.validate()?;
/// # anyhow::Ok(())
/// ```
///
/// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
/// [`validate`メソッド]: Self::validate
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
///
/// # Validation
///
/// この構造体は不正な状態を表現しうる。どのような状態が不正なのかについては[`validate`メソッド]を参照。この構造体を使う関数は、不正な状態に対して[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
///
/// [`Deserialize`]時には、不正な状態であるかの検証は行われない。外部からのデータが不正でないことを確かめるには、デシリアライズ後に`validate`メソッドを用いる必要がある。
///
/// ```
/// # use voicevox_core::AccentPhrase;
/// # let json = r#"{ "moras": [{ "text": "ア", "vowel": "a", "vowel_length": 0, "pitch": 0.0 }], "accent": 1 }"#;
/// let accent_phrase = serde_json::from_str::<AccentPhrase>(json)?;
/// accent_phrase.validate()?;
/// # anyhow::Ok(())
/// ```
///
/// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
/// [`validate`メソッド]: Self::validate
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
/// # Serde
///
/// [Serde]においては[`accent_phrases`]を除くフィールド名はsnake\_caseの形ではなく、VOICEVOX
/// ENGINEに合わせる形でcamelCaseになっている。ただし今後の破壊的変更にて変わる可能性がある。[データのシリアライゼーション]を参照。
///
/// [`accent_phrases`]: Self::accent_phrases
/// [Serde]: serde
/// [データのシリアライゼーション]: https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md
///
/// # Validation
///
/// この構造体は不正な状態を表現しうる。どのような状態が不正なのかについては[`validate`メソッド]を参照。この構造体を使う関数は、不正な状態に対して[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
///
/// [`Deserialize`]時には、不正な状態であるかの検証は行われない。外部からのデータが不正でないことを確かめるには、デシリアライズ後に`validate`メソッドを用いる必要がある。
///
/// ```
/// # use voicevox_core::AudioQuery;
/// # let json = r#"{ "accent_phrases": [], "speedScale": 1.0, "pitchScale": 0.0, "intonationScale": 1.0, "volumeScale": 1.0, "prePhonemeLength": 0.1, "postPhonemeLength": 0.1, "outputSamplingRate": 24000, "outputStereo": false }"#;
/// let audio_query = serde_json::from_str::<AudioQuery>(json)?;
/// audio_query.validate()?;
/// # anyhow::Ok(())
/// ```
///
/// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
/// [`validate`メソッド]: Self::validate
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AudioQuery {
    /// アクセント句の配列。
    pub accent_phrases: Vec<AccentPhrase>,
    /// 全体の話速。
    ///
    /// # Serde
    ///
    /// [Serde]においては`speedScale`という名前で扱われる。
    ///
    /// [Serde]: serde
    #[serde(rename = "speedScale")]
    pub speed_scale: f32,
    /// 全体の音高。
    ///
    /// # Serde
    ///
    /// [Serde]においては`pitchScale`という名前で扱われる。
    ///
    /// [Serde]: serde
    #[serde(rename = "pitchScale")]
    pub pitch_scale: f32,
    /// 全体の抑揚。
    ///
    /// # Serde
    ///
    /// [Serde]においては`intonationScale`という名前で扱われる。
    ///
    /// [Serde]: serde
    #[serde(rename = "intonationScale")]
    pub intonation_scale: f32,
    /// 全体の音量。
    ///
    /// # Serde
    ///
    /// [Serde]においては`volumeScale`という名前で扱われる。
    ///
    /// [Serde]: serde
    #[serde(rename = "volumeScale")]
    pub volume_scale: f32,
    /// 音声の前の無音時間。
    ///
    /// # Serde
    ///
    /// [Serde]においては`prePhonemeLength`という名前で扱われる。
    ///
    /// [Serde]: serde
    #[serde(rename = "prePhonemeLength")]
    pub pre_phoneme_length: f32,
    /// 音声の後の無音時間。
    ///
    /// # Serde
    ///
    /// [Serde]においては`postPhonemeLength`という名前で扱われる。
    ///
    /// [Serde]: serde
    #[serde(rename = "postPhonemeLength")]
    pub post_phoneme_length: f32,
    /// 音声データの出力サンプリングレート。
    ///
    /// # Serde
    ///
    /// [Serde]においては`outputSamplingRate`という名前で扱われる。
    ///
    /// [Serde]: serde
    #[serde(rename = "outputSamplingRate")]
    pub output_sampling_rate: u32,
    /// 音声データをステレオ出力するか否か。
    ///
    /// # Serde
    ///
    /// [Serde]においては`outputStereo`という名前で扱われる。
    ///
    /// [Serde]: serde
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

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::{super::super::DEFAULT_SAMPLING_RATE, AudioQuery};

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
            "outputSamplingRate": DEFAULT_SAMPLING_RATE,
            "outputStereo": false
        }))?;
        Ok(())
    }
}
