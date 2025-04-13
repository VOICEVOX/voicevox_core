use arrayvec::ArrayVec;
use ndarray::Array1;

use crate::{Note, NoteId};

pub(crate) struct ScoreFeature<'score> {
    pub(crate) note_lengths: Array1<i64>,
    pub(crate) note_constants: Array1<i64>,
    pub(crate) note_vowels: Array1<i64>,
    pub(crate) phonemes: Array1<i64>,
    pub(crate) phoneme_keys: Array1<i64>,
    pub(crate) phoneme_note_ids: Vec<Option<&'score NoteId>>,
}

impl<'score> TryFrom<&'score [Note]> for ScoreFeature<'score> {
    type Error = std::convert::Infallible; // TODO

    fn try_from(notes: &'score [Note]) -> std::result::Result<Self, Self::Error> {
        let feature = notes.iter().map(NoteFeature::from).collect::<Vec<_>>();

        let (note_lengths, (note_constants, note_vowels)) = feature
            .iter()
            .map(
                |&NoteFeature {
                     note_length,
                     note_constant,
                     note_vowel,
                     ..
                 }| (note_length, (note_constant, note_vowel)),
            )
            .collect();

        let (phonemes, (phoneme_keys, phoneme_note_ids)) = feature
            .iter()
            .flat_map(|NoteFeature { phonemes, .. }| phonemes)
            .map(
                |&PhonemeFeature {
                     phoneme,
                     phoneme_key,
                     phoneme_note_id,
                 }| (phoneme, (phoneme_key, phoneme_note_id)),
            )
            .collect();

        // FIXME: ndarrayをv0.16に上げれば`Vec`を介する必要がない
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

struct NoteFeature<'score> {
    note_length: i64,
    note_constant: i64,
    note_vowel: i64,
    phonemes: ArrayVec<PhonemeFeature<'score>, 2>, // 1 or 2 phonemes
}

impl<'score> From<&'score Note> for NoteFeature<'score> {
    fn from(
        Note {
            id,
            key,
            frame_length,
            lyric,
        }: &'score Note,
    ) -> Self {
        match &**lyric {
            "" => {
                if key.is_some() {
                    todo!("lyricが空文字列の場合、keyはnullである必要があります。");
                }
                Self {
                    note_length: *frame_length as _, // FIXME
                    note_constant: -1,
                    note_vowel: 0, // pau
                    phonemes: [PhonemeFeature {
                        phoneme: 0, // pau
                        phoneme_key: -1,
                        phoneme_note_id: id.as_ref(),
                    }]
                    .into_iter()
                    .collect(),
                }
            }
            lyric => {
                if key.is_none() {
                    todo!("keyがnullの場合、lyricは空文字列である必要があります。");
                }
                todo!();
            }
        }
    }
}

struct PhonemeFeature<'score> {
    phoneme: i64,
    phoneme_key: i64,
    phoneme_note_id: Option<&'score NoteId>,
}
