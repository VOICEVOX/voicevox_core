use ndarray::Array1;

use crate::{Note, NoteId};

pub(crate) struct ScoreFeature {
    pub(crate) note_lengths: Array1<i64>,
    pub(crate) note_constants: Array1<i64>,
    pub(crate) note_vowels: Array1<i64>,
    pub(crate) phonemes: Array1<i64>,
    pub(crate) phoneme_keys: Array1<i64>,
    pub(crate) phoneme_note_ids: Vec<Option<NoteId>>,
}

impl TryFrom<&'_ [Note]> for ScoreFeature {
    type Error = std::convert::Infallible; // TODO

    fn try_from(notes: &'_ [Note]) -> std::result::Result<Self, Self::Error> {
        let (
            note_lengths,
            (note_constants, (note_vowels, (phonemes, (phoneme_keys, phoneme_note_ids)))),
        ) = notes
            .iter()
            .map(|note| {
                let NoteFeature {
                    note_length,
                    note_constant,
                    note_vowel,
                    phoneme,
                    phoneme_key,
                    phoneme_note_id,
                } = note.into();

                (
                    note_length,
                    (
                        note_constant,
                        (note_vowel, (phoneme, (phoneme_key, phoneme_note_id))),
                    ),
                )
            })
            .collect();

        // FIXME: ndarray v0.16なら`Vec`を介する必要がない
        use std::convert::identity;
        let note_lengths = identity::<Vec<_>>(note_lengths).into();
        let note_constants = identity::<Vec<_>>(note_constants).into();
        let note_vowels = identity::<Vec<_>>(note_vowels).into();
        let phonemes = identity::<Vec<_>>(phonemes).into();
        let phoneme_keys = identity::<Vec<_>>(phoneme_keys).into();

        Ok(Self {
            note_lengths,
            note_constants,
            note_vowels,
            phonemes,
            phoneme_keys,
            phoneme_note_ids,
        })
    }
}

struct NoteFeature {
    note_length: i64,
    note_constant: i64,
    note_vowel: i64,
    phoneme: i64,
    phoneme_key: i64,
    phoneme_note_id: Option<NoteId>,
}

impl From<&'_ Note> for NoteFeature {
    fn from(
        Note {
            id,
            key,
            frame_length,
            lyric,
        }: &'_ Note,
    ) -> Self {
        match &**lyric {
            "" => Self {
                note_length: *frame_length as _, // FIXME
                note_constant: -1,
                note_vowel: 0, // pau
                phoneme: 0,    // pau
                phoneme_key: -1,
                phoneme_note_id: id.clone(),
            },
            lyric => {
                todo!();
            }
        }
    }
}
