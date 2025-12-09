use std::num::NonZero;

use arrayvec::ArrayVec;
use typeshare::U53;

use crate::{
    collections::{NonEmptyIterator, NonEmptyVec},
    error::{ErrorRepr, InvalidQueryError, InvalidQueryErrorSource},
};

use super::{
    super::acoustic_feature_extractor::{MoraTail, OptionalConsonant, PhonemeCode},
    queries::{FramePhoneme, Note, NoteId, OptionalLyric, Score},
};

use self::note_seq::ValidatedNoteSeq;

pub(crate) fn join_frame_phonemes_with_notes<'a>(
    frame_phonemes: &'a [FramePhoneme],
    notes: &'a [ValidatedNote],
) -> Result<
    impl Iterator<Item = (&'a FramePhoneme, &'a ValidatedNote)> + Clone,
    InvalidQueryErrorSource,
> {
    let phonemes_from_query = frame_phonemes
        .iter()
        .map(|p| (PhonemeCode::from(p.phoneme.clone()), p));

    let phonemes_from_score = notes
        .iter()
        .flat_map(|note| note.phonemes().into_iter().map(move |p| (p, note)));

    if !itertools::equal(
        phonemes_from_query.clone().map(|(p, _)| p),
        phonemes_from_score.clone().map(|(p, _)| p),
    ) {
        return Err(InvalidQueryErrorSource::DifferentPhonemeSeqs);
    }

    Ok(itertools::zip_eq(
        phonemes_from_query.map(|(_, p)| p),
        phonemes_from_score.map(|(_, n)| n),
    ))
}

impl Score {
    pub fn validate(&self) -> crate::Result<()> {
        self.to_validated().map(|_| ())
    }

    pub(crate) fn to_validated(&self) -> crate::Result<ValidatedScore> {
        let notes = ValidatedNoteSeq::new(&self.notes)?;
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

        let pau_or_key_and_lyric = PauOrKeyAndLyric::new(key, &lyric)?;

        Ok(ValidatedNote {
            id,
            pau_or_key_and_lyric,
            frame_length,
        })
    }
}

pub(crate) struct ValidatedScore {
    pub(crate) notes: ValidatedNoteSeq,
}

pub(crate) struct ValidatedNote {
    pub(crate) id: Option<NoteId>,
    pub(crate) pau_or_key_and_lyric: PauOrKeyAndLyric,
    pub(crate) frame_length: U53,
}

impl ValidatedNote {
    fn phonemes(&self) -> ArrayVec<PhonemeCode, 2> {
        match self.pau_or_key_and_lyric {
            PauOrKeyAndLyric::Pau => [PhonemeCode::MorablePau].into_iter().collect(),
            // TODO: Rust 1.91以降なら`std::iter::chain`がある
            PauOrKeyAndLyric::KeyAndLyric {
                lyric:
                    Lyric {
                        phonemes: [(consonant, vowel)],
                        ..
                    },
                ..
            } => itertools::chain(consonant.try_into(), [vowel.into()]).collect(),
        }
    }
}

#[derive(PartialEq)]
pub(crate) enum PauOrKeyAndLyric {
    Pau,
    KeyAndLyric { key: U53, lyric: Lyric },
}

impl PauOrKeyAndLyric {
    fn new(key: Option<U53>, lyric: &OptionalLyric) -> crate::Result<Self> {
        match (key, &*lyric.phonemes) {
            (None, []) => Ok(Self::Pau),
            (Some(key), &[mora]) => Ok(Self::KeyAndLyric {
                key,
                lyric: Lyric { phonemes: [mora] },
            }),
            (Some(_), []) => Err(ErrorRepr::InvalidQuery(InvalidQueryError {
                what: "ノート",
                value: None,
                source: Some(InvalidQueryErrorSource::UnnecessaryKeyForPau),
            })
            .into()),
            (None, [_]) => Err(ErrorRepr::InvalidQuery(InvalidQueryError {
                what: "ノート",
                value: None,
                source: Some(InvalidQueryErrorSource::MissingKeyForNonPau),
            })
            .into()),
            (_, [_, ..]) => unreachable!("the lyric should consist of at most one mora"),
        }
    }
}

#[derive(PartialEq)]
pub(crate) struct Lyric {
    // TODO: `NonPauBaseVowel`型 (= a | i | u | e | o | cl | N) を導入する
    pub(super) phonemes: [(OptionalConsonant, MoraTail); 1],
}

impl ValidatedNoteSeq {
    pub(crate) fn new(notes: &[Note]) -> crate::Result<Self> {
        let notes = notes
            .iter()
            .cloned()
            .map(Note::into_validated)
            .collect::<Result<Vec<_>, _>>()?;

        NonEmptyVec::new(notes)
            .ok_or_else(|| InvalidQueryError {
                what: "ノート列",
                value: None,
                source: Some(InvalidQueryErrorSource::InitialNoteMustBePau),
            })?
            .try_into()
    }

    pub(crate) fn len(&self) -> NonZero<usize> {
        AsRef::<NonEmptyVec<_>>::as_ref(self).len()
    }

    pub(crate) fn iter(&self) -> impl NonEmptyIterator<Item = &ValidatedNote> {
        AsRef::<NonEmptyVec<_>>::as_ref(self).iter()
    }
}

impl AsRef<[ValidatedNote]> for ValidatedNoteSeq {
    fn as_ref(&self) -> &[ValidatedNote] {
        AsRef::<NonEmptyVec<_>>::as_ref(self).as_ref()
    }
}

pub(crate) mod note_seq {
    use derive_more::AsRef;

    use crate::{
        collections::NonEmptyVec,
        error::{ErrorRepr, InvalidQueryError, InvalidQueryErrorSource},
    };

    use super::{PauOrKeyAndLyric, ValidatedNote};

    #[derive(AsRef)]
    pub(crate) struct ValidatedNoteSeq(
        NonEmptyVec<ValidatedNote>, // invariant: the first note must be pau
    );

    impl TryFrom<NonEmptyVec<ValidatedNote>> for ValidatedNoteSeq {
        type Error = crate::Error;

        fn try_from(notes: NonEmptyVec<ValidatedNote>) -> Result<Self, Self::Error> {
            if notes.first().pau_or_key_and_lyric != PauOrKeyAndLyric::Pau {
                return Err(ErrorRepr::InvalidQuery(InvalidQueryError {
                    what: "ノート列",
                    value: None,
                    source: Some(InvalidQueryErrorSource::InitialNoteMustBePau),
                })
                .into());
            }
            Ok(Self(notes))
        }
    }
}
