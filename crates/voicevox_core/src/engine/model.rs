use std::fmt;

use duplicate::duplicate_item;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

/* 各フィールドのjsonフィールド名はsnake_caseとする*/

/// モーラ（子音＋母音）ごとの情報。
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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
#[derive(Clone, Deserialize, Serialize)]
pub struct AudioQuery {
    /// アクセント句の配列。
    pub accent_phrases: Vec<AccentPhrase>,
    /// 全体の話速。
    pub speed_scale: f32,
    /// 全体の音高。
    pub pitch_scale: f32,
    /// 全体の抑揚。
    pub intonation_scale: f32,
    /// 全体の音量。
    pub volume_scale: f32,
    /// 音声の前の無音時間。
    pub pre_phoneme_length: f32,
    /// 音声の後の無音時間。
    pub post_phoneme_length: f32,
    /// 音声データの出力サンプリングレート。
    pub output_sampling_rate: u32,
    /// 音声データをステレオ出力するか否か。
    pub output_stereo: bool,
    // TODO: VOICEVOX/voicevox_engine#1308 を実装する
    /// 句読点などの無音時間。`null`のときは無視される。デフォルト値は`null`。
    #[serde(
        default,
        deserialize_with = "deserialize_pause_length",
        serialize_with = "serialize_pause_length"
    )]
    pub pause_length: (),
    /// 読点などの無音時間（倍率）。デフォルト値は`1`。
    #[serde(
        default,
        deserialize_with = "deserialize_pause_length_scale",
        serialize_with = "serialize_pause_length_scale"
    )]
    pub pause_length_scale: (),
    /// \[読み取り専用\] AquesTalk風記法。
    ///
    /// [`Synthesizer::audio_query`]が返すもののみ`Some`となる。入力としてのAudioQueryでは無視され
    /// る。
    ///
    /// [`Synthesizer::audio_query`]: crate::blocking::Synthesizer::audio_query
    pub kana: Option<String>,
}

fn deserialize_pause_length<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    return deserializer.deserialize_any(Visitor);

    struct Visitor;

    impl de::Visitor<'_> for Visitor {
        type Value = ();

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("`null`")
        }

        #[duplicate_item(
            method        T;
            [ visit_i64 ] [ i64 ];
            [ visit_u64 ] [ u64 ];
            [ visit_f64 ] [ f64 ];
        )]
        fn method<E>(self, _: T) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Err(E::custom("currently `pause_length` must be `null`"))
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(())
        }
    }
}

fn serialize_pause_length<S>(_: &(), serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_unit()
}

fn deserialize_pause_length_scale<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    return deserializer.deserialize_any(Visitor);

    struct Visitor;

    impl de::Visitor<'_> for Visitor {
        type Value = ();

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("`1.`")
        }

        #[duplicate_item(
            method        T       ONE;
            [ visit_i64 ] [ i64 ] [ 1 ];
            [ visit_u64 ] [ u64 ] [ 1 ];
            [ visit_f64 ] [ f64 ] [ 1. ];
        )]
        fn method<E>(self, v: T) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v != ONE {
                return Err(E::custom("currently `pause_length_scale` must be `1.`"));
            }
            Ok(())
        }
    }
}

fn serialize_pause_length_scale<S>(_: &(), serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    (1.).serialize(serializer)
}

impl AudioQuery {
    pub(crate) fn with_kana(self, kana: Option<String>) -> Self {
        Self { kana, ..self }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use serde_json::json;

    use super::AudioQuery;

    #[rstest]
    fn check_audio_query_model_json_field_snake_case() {
        let audio_query_model = AudioQuery {
            accent_phrases: vec![],
            speed_scale: 0.0,
            pitch_scale: 0.0,
            intonation_scale: 0.0,
            volume_scale: 0.0,
            pre_phoneme_length: 0.0,
            post_phoneme_length: 0.0,
            output_sampling_rate: 0,
            output_stereo: false,
            pause_length: (),
            pause_length_scale: (),
            kana: None,
        };
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

    // TODO: 型的に自明になったらこのテストは削除する
    #[rstest]
    fn it_denies_non_null_for_pause_length() {
        serde_json::from_value::<AudioQuery>(json!({
            "accent_phrases": [],
            "speed_scale": 1.0,
            "pitch_scale": 0.0,
            "intonation_scale": 1.0,
            "volume_scale": 1.0,
            "pre_phoneme_length": 0.1,
            "post_phoneme_length": 0.1,
            "output_sampling_rate": 24000,
            "output_stereo": false,
            "pause_length": "aaaaa"
        }))
        .map(|_| ())
        .unwrap_err();
    }

    // TODO: 型的に自明になったらこのテストは削除する
    #[rstest]
    fn it_denies_non_float_for_pause_length_scale() {
        serde_json::from_value::<AudioQuery>(json!({
            "accent_phrases": [],
            "speed_scale": 1.0,
            "pitch_scale": 0.0,
            "intonation_scale": 1.0,
            "volume_scale": 1.0,
            "pre_phoneme_length": 0.1,
            "post_phoneme_length": 0.1,
            "output_sampling_rate": 24000,
            "output_stereo": false,
            "pause_length_scale": "aaaaa",
        }))
        .map(|_| ())
        .unwrap_err();
    }
}
