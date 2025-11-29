mod validated;

use std::{num::NonZero, sync::Arc};

use serde::{Deserialize, Serialize};
use typed_floats::{NonNaNFinite, PositiveFinite};
use typeshare::U53;

use super::super::Phoneme;

pub(crate) use self::validated::{KeyAndLyric, ValidatedNote};

/// 音符のID。
#[derive(Clone, Deserialize, Serialize)]
pub struct NoteId(pub Arc<str>);

/// 音符ごとの情報。
#[derive(Clone)]
pub struct Note {
    /// ID。
    pub id: Option<NoteId>,

    /// 音階。
    pub key: Option<U53>,

    /// 歌詞。
    pub lyric: String,

    /// 音符のフレーム長。
    pub frame_length: U53,
}

/// 楽譜情報。
#[derive(Clone)]
pub struct Score {
    /// 音符のリスト。
    pub notes: Vec<Note>,
}

/// 音素の情報。
#[derive(Clone, Deserialize, Serialize)]
pub struct FramePhoneme {
    /// 音素。
    pub phoneme: Phoneme,

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
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameAudioQuery {
    /// フレームごとの基本周波数。
    pub f0: Vec<NonNaNFinite<f32>>,

    /// フレームごとの音量。
    pub volume: Vec<PositiveFinite<f32>>,

    /// 音素のリスト。
    pub phonemes: Vec<FramePhoneme>,

    /// 全体の音量。
    pub volume_scale: PositiveFinite<f32>,

    /// 音声データの出力サンプリングレート。
    pub output_sample_rate: NonZero<u32>,

    /// 音声データをステレオ出力するか否か。
    pub output_stereo: bool,
}
