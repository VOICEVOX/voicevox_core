use arrayvec::ArrayVec;
use duplicate::duplicate;
use easy_ext::ext;
use ndarray::Array1;
use pastey::paste;
use typeshare::U53;

use crate::NoteId;

use super::ValidatedNote;

pub(crate) struct ScoreFeature<'score> {
    pub(crate) note_lengths: Array1<i64>,
    pub(crate) note_constants: Array1<i64>,
    pub(crate) note_vowels: Array1<i64>,
    pub(crate) phonemes: Array1<i64>,
    pub(crate) phoneme_keys: Array1<i64>,
    pub(crate) phoneme_note_ids: Vec<Option<&'score NoteId>>,
}

impl<'score> From<&'score [ValidatedNote]> for ScoreFeature<'score> {
    fn from(notes: &'score [ValidatedNote]) -> Self {
        let feature = notes.iter().map(Into::into).collect::<Vec<_>>();

        duplicate! {
            [
                x;
                [ note_length ];
                [ note_constant ];
                [ note_vowel ];
            ]
            let paste! { [<x s>] } = feature
                .iter()
                .map(|&NoteFeature { x, .. }| x)
                .collect();
        }

        let phoneme_features = feature
            .iter()
            .flat_map(|NoteFeature { phonemes, .. }| phonemes)
            .copied()
            .collect::<Vec<_>>();

        duplicate! {
            [
                x;
                [ phoneme ];
                [ phoneme_key ];
                [ phoneme_note_id ];
            ]
            let paste! { [<x s>] } = phoneme_features
                .iter()
                .map(|&PhonemeFeature { x, .. }| x)
                .collect();
        }

        Self {
            note_lengths,
            note_constants,
            note_vowels,
            phonemes,
            phoneme_keys,
            phoneme_note_ids,
        }
    }
}

struct NoteFeature<'score> {
    note_length: i64,
    note_constant: i64,
    note_vowel: i64,
    phonemes: ArrayVec<PhonemeFeature<'score>, 2>, // 1 or 2 phonemes
}

impl<'score> From<&'score ValidatedNote> for NoteFeature<'score> {
    fn from(
        ValidatedNote {
            id,
            key_and_lyric,
            frame_length,
        }: &'score ValidatedNote,
    ) -> Self {
        if let Some(key_and_lyric) = key_and_lyric {
            todo!();
        } else {
            Self {
                note_length: frame_length.to_i64(),
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
    }
}

#[derive(Clone, Copy)]
struct PhonemeFeature<'score> {
    phoneme: i64,
    phoneme_key: i64,
    phoneme_note_id: Option<&'score NoteId>,
}

#[ext]
impl U53 {
    fn to_i64(self) -> i64 {
        u64::from(self).try_into().expect("this is 53-bit")
    }
}
