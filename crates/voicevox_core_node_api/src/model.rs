use napi::bindgen_prelude::Array;

#[napi(object)]
pub struct Mora {
    /// 文字。
    pub text: String,
    /// 子音の音素。
    pub consonant: Option<String>,
    /// 子音の音長。
    pub consonant_length: Option<f64>,
    /// 母音の音素。
    pub vowel: String,
    /// 母音の音長。
    pub vowel_length: f64,
    /// 音高。
    pub pitch: f64,
}

#[napi(object)]
pub struct AccentPhrase {
    /// モーラの配列。
    #[napi(ts_type = "Mora[]")]
    pub moras: Array,
    /// アクセント箇所。
    pub accent: i64,
    /// 後ろに無音を付けるかどうか。
    pub pause_mora: Option<Mora>,
    /// 疑問系かどうか。
    pub is_interrogative: bool,
}

#[napi(object)]
pub struct AudioQuery {
    /// アクセント句の配列。
    #[napi(ts_type = "AccentPhrase[]")]
    pub accent_phrases: Array,
    /// 全体の話速。
    pub speed_scale: f64,
    /// 全体の音高。
    pub pitch_scale: f64,
    /// 全体の抑揚。
    pub intonation_scale: f64,
    /// 全体の音量。
    pub volume_scale: f64,
    /// 音声の前の無音時間。
    pub pre_phoneme_length: f64,
    /// 音声の後の無音時間。
    pub post_phoneme_length: f64,
    /// 音声データの出力サンプリングレート。
    pub output_sampling_rate: u32,
    /// 音声データをステレオ出力するか否か。
    pub output_stereo: bool,
    /// \[読み取り専用\] AquesTalk風記法。
    ///
    /// [`Synthesizer::audio_query`]が返すもののみ`Some`となる。入力としてのAudioQueryでは無視され
    /// る。
    ///
    /// [`Synthesizer::audio_query`]: crate::Synthesizer::audio_query
    pub kana: Option<String>,
}
