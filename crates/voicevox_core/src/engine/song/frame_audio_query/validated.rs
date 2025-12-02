use std::num::NonZero;

use arrayvec::ArrayVec;
use typeshare::U53;

use crate::{
    error::{ErrorRepr, InvalidQueryErrorKind},
    FramePhoneme,
};

use super::{
    super::super::acoustic_feature_extractor::{MoraTail, OptionalConsonant, PhonemeCode},
    Note, NoteId, OptionalLyric, Score,
};

impl Score {
    pub fn validate(&self) -> crate::Result<()> {
        self.notes.iter().try_for_each(Note::validate)
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
    fn phonemes(&self) -> ArrayVec<PhonemeCode, 2> {
        if let Some(KeyAndLyric {
            lyric:
                Lyric {
                    phonemes: [(consonant, vowel)],
                    ..
                },
            ..
        }) = self.key_and_lyric
        {
            // TODO: Rust 1.91以降なら`std::iter::chain`がある
            itertools::chain(consonant.try_into(), [vowel.into()]).collect()
        } else {
            [PhonemeCode::MorablePau].into_iter().collect()
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
                lyric: Lyric { phonemes: [mora] },
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
    pub(in super::super) phonemes: [(OptionalConsonant, MoraTail); 1],
}

pub(crate) struct ValidatedNoteSeqWithConsonantLengths {
    //pub(crate) notes: ValidatedNoteSeq,
    pub(crate) phoneme_lengths: Vec<usize>,
}

impl ValidatedNoteSeqWithConsonantLengths {
    pub(crate) fn new(notes: ValidatedNoteSeq, phonemes: &[FramePhoneme]) -> crate::Result<Self> {
        let phonemes_from_score = notes.iter().flat_map(ValidatedNote::phonemes);
        let phonemes_from_query = phonemes
            .iter()
            .map(|FramePhoneme { phoneme, .. }| PhonemeCode::from(phoneme.clone()));
        if !itertools::equal(phonemes_from_score, phonemes_from_query) {
            todo!();
        }
        Ok(Self {
            //notes,
            phoneme_lengths: phonemes
                .iter()
                .map(|&FramePhoneme { frame_length, .. }| {
                    typeshare::usize_from_u53_saturated(frame_length)
                })
                .collect(),
        })
    }
}
