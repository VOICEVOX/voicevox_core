use arrayvec::ArrayVec;
use duplicate::duplicate;
use easy_ext::ext;
use ndarray::Array1;
use pastey::paste;
use typeshare::U53;

use crate::{
    collections::{NonEmptyIterator as _, NonEmptySlice},
    NoteId,
};

use super::{
    super::acoustic_feature_extractor::{OptionalConsonant, PhonemeCode},
    frame_audio_query::{Lyric, PauOrKeyAndLyric, ValidatedNote, ValidatedNoteSeq},
};

pub(crate) struct ScoreFeature<'score> {
    pub(crate) note_lengths: Array1<i64>,
    pub(crate) note_constants: Array1<i64>,
    pub(crate) note_vowels: Array1<i64>,
    pub(crate) phonemes: Vec<PhonemeCode>,
    pub(crate) phoneme_keys: Array1<i64>,
    pub(crate) phoneme_note_ids: Vec<Option<&'score NoteId>>,
}

impl<'score> From<&'score ValidatedNoteSeq> for ScoreFeature<'score> {
    fn from(notes: &'score ValidatedNoteSeq) -> Self {
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
            pau_or_key_and_lyric,
            frame_length,
        }: &'score ValidatedNote,
    ) -> Self {
        match *pau_or_key_and_lyric {
            PauOrKeyAndLyric::Pau => Self {
                note_length: frame_length.to_i64(),
                note_constant: OptionalConsonant::None as _,
                note_vowel: PhonemeCode::MorablePau as _,
                phonemes: [PhonemeFeature {
                    phoneme: PhonemeCode::MorablePau,
                    phoneme_key: -1,
                    phoneme_note_id: id.as_ref(),
                }]
                .into_iter()
                .collect(),
            },
            PauOrKeyAndLyric::KeyAndLyric {
                key,
                lyric:
                    Lyric {
                        phonemes: [(consonant, vowel)],
                        ..
                    },
            } => Self {
                note_length: frame_length.to_i64(),
                note_constant: consonant as _,
                note_vowel: vowel as _,
                // TODO: Rust 1.91以降なら`std::iter::chain`がある
                phonemes: itertools::chain(
                    consonant.try_into().map(|phoneme| PhonemeFeature {
                        phoneme,
                        phoneme_key: key.to_i64(),
                        phoneme_note_id: id.as_ref(),
                    }),
                    [PhonemeFeature {
                        phoneme: vowel.into(),
                        phoneme_key: key.to_i64(),
                        phoneme_note_id: id.as_ref(),
                    }],
                )
                .collect(),
            },
        }
    }
}

#[derive(Clone, Copy)]
struct PhonemeFeature<'score> {
    phoneme: PhonemeCode,
    phoneme_key: i64,
    phoneme_note_id: Option<&'score NoteId>,
}

pub(crate) fn phoneme_lengths(
    consonant_lengths: &NonEmptySlice<i64>,
    note_durations: &NonEmptySlice<U53>,
) -> Vec<U53> {
    if consonant_lengths.len() != note_durations.len() {
        panic!("must be same length");
    }

    let (&first_consonant_length, next_consonant_lengths) = consonant_lengths.split_first();

    if first_consonant_length != 0 {
        panic!("`consonant_lengths[0]` cannot be non-zero");
    }

    let (&last_note_duration, note_durations_till_last) = note_durations.split_last();

    let next_consonant_lengths = &{
        let mut next_consonant_lengths = next_consonant_lengths.to_owned();
        for (next_consonant_length, &note_duration) in
            itertools::zip_eq(&mut next_consonant_lengths, note_durations_till_last)
        {
            // 次のノートの子音長 (`next_consonant_length`)が以下の条件を満たすなら、
            // 現在のノート長 (`note_duration`)の半分の値に置き換える。
            //
            // - 負
            // - 現在のノート長を超過する
            if next_consonant_length.is_negative()
                || note_duration.to_i64() < *next_consonant_length
            {
                *next_consonant_length = note_duration.to_i64() / 2;
            }
        }
        next_consonant_lengths
    };

    assert!(
        next_consonant_lengths.iter().any(|&n| n >= 0),
        "elements should have been replaced with non-negative values",
    );
    let next_consonant_lengths = bytemuck::must_cast_slice::<_, u64>(next_consonant_lengths);

    itertools::zip_eq(next_consonant_lengths, note_durations_till_last)
        .flat_map(|(&next_consonant_length, &note_duration)| {
            let note_duration = u64::from(note_duration);
            let vowel_length = note_duration
                .checked_sub(next_consonant_length)
                .expect("each `next_consonant_length` should have been replaced with small values")
                .try_into()
                .expect("should equal or be smaller than `note_duration`");
            // TODO: Rust 1.91以降なら`std::iter::chain`がある
            itertools::chain(
                [vowel_length],
                (next_consonant_length > 0).then(|| {
                    next_consonant_length
                        .try_into()
                        .unwrap_or_else(|_| unimplemented!("too large"))
                }),
            )
        })
        .chain([last_note_duration])
        .collect()
}

#[ext]
impl U53 {
    fn to_i64(self) -> i64 {
        u64::from(self).try_into().expect("this is 53-bit")
    }
}
