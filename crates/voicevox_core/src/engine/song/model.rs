use serde::{Deserialize, Serialize};
use typeshare::U53;

/// 音符のID。
#[derive(Clone, Deserialize, Serialize)]
pub struct NoteId(pub String);

/// 音符ごとの情報。
pub struct Note {
    /// ID。
    pub id: Option<NoteId>,

    /// 音階と歌詞。
    pub key_and_lyric: KeyAndLyric,

    /// 音符のフレーム長。
    pub frame_length: U53,
}

/// 音階と歌詞。
pub struct KeyAndLyric {
    key: Option<U53>,
    lyric: String,
}

impl KeyAndLyric {
    pub fn new(key: Option<U53>, lyric: String) -> Result<Self, std::convert::Infallible> {
        if key.is_some() && lyric.is_empty() {
            todo!("lyricが空文字列の場合、keyはnullである必要があります。");
        }
        if key.is_none() && !lyric.is_empty() {
            todo!("keyがnullの場合、lyricは空文字列である必要があります。");
        }
        Ok(Self { key, lyric })
    }

    /// 音階。
    pub fn key(&self) -> Option<U53> {
        self.key
    }

    /// 音符の歌詞。
    pub fn lyric(&self) -> &str {
        &self.lyric
    }
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
    pub frame_length: U53,

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
    pub output_sample_rate: u32,

    /// 音声データをステレオ出力するか否か。
    pub output_stereo: bool,
}
