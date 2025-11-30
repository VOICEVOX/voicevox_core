use std::num::NonZero;

use smol_str::SmolStr;
use typed_floats::PositiveFinite;
use typeshare::U53;

use super::{
    super::super::acoustic_feature_extractor::{MoraTail, NonPauPhonemeCode, OptionalConsonant},
    Note, NoteId, OptionalLyric, Score,
};

impl Score {
    pub fn validte(&self) -> crate::Result<()> {
        self.clone().into_validated().map(|_| ())
    }

    pub(crate) fn into_validated(self) -> crate::Result<ValidatedScore> {
        let notes = self
            .notes
            .into_iter()
            .map(Note::into_validated)
            .collect::<Result<_, _>>()?;
        Ok(ValidatedScore { notes })
    }
}

impl Note {
    pub fn validate(&self) -> crate::Result<()> {
        self.clone().into_validated().map(|_| ())
    }

    fn into_validated(self) -> crate::Result<ValidatedNote> {
        let Self {
            id,
            key,
            lyric,
            frame_length,
        } = self;

        let key_and_lyric = KeyAndLyric::new(key, &lyric)?;

        Ok(ValidatedNote {
            id,
            key_and_lyric,
            frame_length,
        })
    }
}

pub(crate) struct ValidatedScore {
    pub(crate) notes: Vec<ValidatedNote>,
}

pub(crate) struct ValidatedNote {
    /// ID。
    pub(crate) id: Option<NoteId>,

    /// 音階と歌詞。
    pub(crate) key_and_lyric: Option<KeyAndLyric>,

    /// 音符のフレーム長。
    pub(crate) frame_length: U53,
}

/// 音階と歌詞。
pub(crate) struct KeyAndLyric {
    pub(in super::super) key: U53,
    pub(in super::super) lyric: Lyric,
}

impl KeyAndLyric {
    fn new(key: Option<U53>, lyric: &OptionalLyric) -> crate::Result<Option<Self>> {
        if key.is_some() && lyric.text.is_empty() {
            todo!("lyricが空文字列の場合、keyはnullである必要があります。");
        }
        if key.is_none() && !lyric.text.is_empty() {
            todo!("keyがnullの場合、lyricは空文字列である必要があります。");
        }
        todo!();
    }
}

pub(in super::super) struct Lyric {
    // invariant: `phonemes` must come from `text`.
    text: SmolStr,
    pub(in super::super) phonemes: [(OptionalConsonant, MoraTail); 1],
}

impl Lyric {
    fn new(optional: &OptionalLyric) -> Option<Self> {
        let phonemes = optional.phonemes.clone().into_inner().ok()?;
        Some(Self {
            text: optional.text.clone(),
            phonemes,
        })
    }
}

pub(crate) struct ContextedFrameAudioQuery {
    pub(crate) f0: Vec<PositiveFinite<f32>>,
    pub(crate) volume: Vec<PositiveFinite<f32>>,
    pub(crate) head: ContextedPauNote,
    pub(crate) tail: Vec<ContextedNote>,
    pub(crate) volume_scale: PositiveFinite<f32>,
    pub(crate) output_sample_rate: NonZero<u32>,
    pub(crate) output_stereo: bool,
}

pub(crate) enum ContextedNote {
    Pau(ContextedPauNote),
    NonPau(ContextedPauNote),
}

pub(crate) struct ContextedPauNote {
    note_id: Option<NoteId>,
    frame_length: U53,
}

pub(crate) struct ContextedNonPauNote {
    key: U53,
    lyric: String,
    note_id: Option<NoteId>,
    phonemes: Vec<LengthedNonPauPhoneme>,
}

#[derive(Clone, Copy)]
pub(crate) struct LengthedNonPauPhoneme {
    phoneme: NonPauPhonemeCode,
    frame_length: U53,
}
