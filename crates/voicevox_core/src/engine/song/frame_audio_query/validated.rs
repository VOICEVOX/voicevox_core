use std::num::NonZero;

use smol_str::SmolStr;
use typed_floats::PositiveFinite;
use typeshare::U53;

use crate::{
    error::{ErrorRepr, InvalidQueryErrorKind},
    SamplingRate,
};

use super::{
    super::super::acoustic_feature_extractor::{MoraTail, NonPauPhonemeCode, OptionalConsonant},
    Note, NoteId, OptionalLyric, Score,
};

impl Score {
    pub fn validte(&self) -> crate::Result<()> {
        self.clone().into_validated().map(|_| ())
    }

    pub(crate) fn into_validated(self) -> crate::Result<ValidatedScore> {
        let notes = ValidatedNoteSeq::new(self.notes)?;
        Ok(ValidatedScore { notes })
    }
}

impl Note {
    pub fn validate(&self) -> crate::Result<()> {
        self.clone().into_validated().map(|_| ())
    }

    pub(crate) fn into_validated(self) -> crate::Result<ValidatedNote> {
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
    notes: ValidatedNoteSeq,
}

// TODO: nonempty-collectionを導入
pub(crate) struct ValidatedNoteSeq {
    pub(in super::super) initial_pau: ValidatedNote, // invariant: must be a pau
    pub(in super::super) rest_notes: Vec<ValidatedNote>,
}

impl ValidatedNoteSeq {
    pub(crate) fn new(notes: impl IntoIterator<Item = Note>) -> crate::Result<Self> {
        let mut notes = notes.into_iter();

        let initial_pau = {
            let error = || {
                ErrorRepr::InvalidQuery {
                    what: "ノート列",
                    kind: InvalidQueryErrorKind::InitialNoteMustBePau,
                }
                .into()
            };
            let head = notes.next().ok_or_else(error)?.into_validated()?;
            if head.key_and_lyric.is_some() {
                return Err(error());
            }
            head
        };

        // TODO: `what`を"ノート"から"ノート列"に置き換える
        let rest_notes = notes.map(Note::into_validated).collect::<Result<_, _>>()?;

        Ok(Self {
            initial_pau,
            rest_notes,
        })
    }
}

impl ValidatedNoteSeq {
    pub(crate) fn len(&self) -> NonZero<usize> {
        NonZero::new(1 + self.rest_notes.len()).expect("")
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &ValidatedNote> {
        // TODO: Rust 1.91以降なら`std::iter::chain`がある
        itertools::chain([&self.initial_pau], &self.rest_notes)
    }
}

pub(crate) struct ValidatedNote {
    /// ID。
    pub(crate) id: Option<NoteId>,

    /// 音階と歌詞。
    pub(crate) key_and_lyric: Option<KeyAndLyric>,

    /// 音符のフレーム長。
    pub(crate) frame_length: U53,
}

impl ValidatedNote {
    pub(in super::super) fn pau(frame_length: U53) -> Self {
        Self {
            id: None,
            key_and_lyric: None,
            frame_length,
        }
    }
}

/// 音階と歌詞。
pub(crate) struct KeyAndLyric {
    pub(in super::super) key: U53,
    pub(in super::super) lyric: Lyric,
}

impl KeyAndLyric {
    fn new(key: Option<U53>, lyric: &OptionalLyric) -> crate::Result<Option<Self>> {
        match (key, &*lyric.phonemes) {
            (None, []) => Ok(None),
            (Some(key), &[mora]) => Ok(Some(Self {
                key,
                lyric: Lyric {
                    text: lyric.text.clone(),
                    phonemes: [mora],
                },
            })),
            (Some(_), []) => Err(ErrorRepr::InvalidQuery {
                what: "ノート",
                kind: InvalidQueryErrorKind::UnnecessaryKeyForPau,
            }
            .into()),
            (None, [_]) => Err(ErrorRepr::InvalidQuery {
                what: "ノート",
                kind: InvalidQueryErrorKind::MissingKeyForNonPau,
            }
            .into()),
            (_, [_, ..]) => unreachable!(),
        }
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
    pub(crate) output_sample_rate: SamplingRate,
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
