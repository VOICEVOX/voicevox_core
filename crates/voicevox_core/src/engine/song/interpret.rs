use std::iter;

use arrayvec::ArrayVec;
use ndarray::Array1;
use typeshare::U53;

use crate::{
    collections::{NonEmptyIterator as _, NonEmptySlice},
    numerics::U53Ext as _,
    FrameAudioQuery, FramePhoneme, NoteId,
};

use super::{
    super::{
        acoustic_feature_extractor::{OptionalConsonant, PhonemeCode},
        ndarray::IteratorExt as _,
    },
    validate::{note_seq::ValidatedNoteSeq, Lyric, PauOrKeyAndLyric, ValidatedNote},
};

/// 子音長と音符長から音素長を計算する。
///
/// 子音はノートの頭にくるようにするため、予測された子音長は前のノートの長さを超えないように調整される。
///
/// 具体的にはi番目のノートの子音長が以下の条件を満たすなら、i-1番目のノート長の半分の値に置き換える。
///
/// - 負
/// - i-1番目のノート長を超過する
///
/// # Panics
///
/// 以下の条件を満たすときパニックする。
///
/// - `consonant_lengths.len() != note_durations.len()`
/// - `consonant_lengths`の先頭が`0`以外
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

    let next_consonant_lengths =
        itertools::zip_eq(next_consonant_lengths, note_durations_till_last).map(
            |(&next_consonant_length, &note_duration)| {
                if next_consonant_length == 0 {
                    None
                } else if next_consonant_length.is_negative()
                    || note_duration.to_i64() < next_consonant_length
                {
                    Some(u64::from(note_duration) / 2)
                } else {
                    Some(u64::try_from(next_consonant_length).expect("should be positive"))
                }
            },
        );

    itertools::zip_eq(next_consonant_lengths, note_durations_till_last)
        .flat_map(|(next_consonant_length, &note_duration)| {
            let note_duration = u64::from(note_duration);
            let vowel_length = note_duration
                .checked_sub(next_consonant_length.unwrap_or(0))
                .expect("each `next_consonant_length` should have been replaced with small values")
                .try_into()
                .expect("should equal or be smaller than `note_duration`");
            // TODO: Rust 1.91以降なら`std::iter::chain`がある
            itertools::chain(
                [vowel_length],
                next_consonant_length.map(|next_consonant_length| {
                    next_consonant_length
                        .try_into()
                        .unwrap_or_else(|_| unimplemented!("too large"))
                }),
            )
        })
        .chain([last_note_duration])
        .collect()
}

pub(crate) fn repeat_phoneme_code_and_key(
    FramePhoneme {
        phoneme,
        frame_length,
        ..
    }: &FramePhoneme,
    ValidatedNote {
        pau_or_key_and_lyric,
        ..
    }: &ValidatedNote,
) -> impl Iterator<Item = (i64, i64)> {
    let phoneme = PhonemeCode::from(phoneme.clone()) as _;
    let key = pau_or_key_and_lyric.key();
    let n = typeshare::usize_from_u53_saturated(*frame_length);
    iter::repeat_n((phoneme, key), n)
}

impl PauOrKeyAndLyric {
    fn key(&self) -> i64 {
        match *self {
            Self::Pau => -1,
            Self::KeyAndLyric { key, .. } => key.to_i64(),
        }
    }
}

impl FrameAudioQuery {
    pub(crate) fn total_frame_length(&self) -> usize {
        self.phonemes
            .iter()
            .map(|&FramePhoneme { frame_length, .. }| {
                typeshare::usize_from_u53_saturated(frame_length)
            })
            .sum()
    }
}

impl ValidatedNoteSeq {
    pub(crate) fn total_frame_length(&self) -> usize {
        self.iter()
            .map(|&ValidatedNote { frame_length, .. }| {
                typeshare::usize_from_u53_saturated(frame_length)
            })
            .sum()
    }
}

pub(crate) struct ConsonantLengthsFeature {
    pub(crate) note_lengths: Array1<i64>,
    pub(crate) note_constants: Array1<i64>,
    pub(crate) note_vowels: Array1<i64>,
}

impl From<&'_ ValidatedNoteSeq> for ConsonantLengthsFeature {
    fn from(notes: &'_ ValidatedNoteSeq) -> Self {
        let (note_lengths, note_constants, note_vowels) = notes
            .iter()
            .into_iter()
            .map(from_note)
            .multiunzip_into_array1s();

        return Self {
            note_lengths,
            note_constants,
            note_vowels,
        };

        fn from_note(
            ValidatedNote {
                pau_or_key_and_lyric,
                frame_length,
                ..
            }: &ValidatedNote,
        ) -> (i64, i64, i64) {
            match *pau_or_key_and_lyric {
                PauOrKeyAndLyric::Pau => (
                    frame_length.to_i64(),
                    OptionalConsonant::None as _,
                    PhonemeCode::MorablePau as _,
                ),
                PauOrKeyAndLyric::KeyAndLyric {
                    lyric:
                        Lyric {
                            phonemes: [(consonant, vowel)],
                            ..
                        },
                    ..
                } => (frame_length.to_i64(), consonant as _, vowel as _),
            }
        }
    }
}

pub(crate) struct PhonemeFeature {
    pub(crate) phoneme: PhonemeCode,
    pub(crate) note_id: Option<NoteId>,
}

impl From<&'_ ValidatedNoteSeq> for Vec<PhonemeFeature> {
    fn from(notes: &'_ ValidatedNoteSeq) -> Self {
        return notes.iter().into_iter().flat_map(from_note).collect();

        fn from_note(
            ValidatedNote {
                id,
                pau_or_key_and_lyric,
                ..
            }: &ValidatedNote,
        ) -> ArrayVec<PhonemeFeature, 2> {
            match *pau_or_key_and_lyric {
                PauOrKeyAndLyric::Pau => [PhonemeFeature {
                    phoneme: PhonemeCode::MorablePau,
                    note_id: id.clone(),
                }]
                .into_iter()
                .collect(),

                // TODO: Rust 1.91以降なら`std::iter::chain`がある
                PauOrKeyAndLyric::KeyAndLyric {
                    lyric:
                        Lyric {
                            phonemes: [(consonant, vowel)],
                            ..
                        },
                    ..
                } => itertools::chain(
                    consonant.try_into().map(|phoneme| PhonemeFeature {
                        phoneme,
                        note_id: id.clone(),
                    }),
                    [PhonemeFeature {
                        phoneme: vowel.into(),
                        note_id: id.clone(),
                    }],
                )
                .collect(),
            }
        }
    }
}

pub(crate) struct SfDecoderFeature {
    pub(crate) frame_phonemes: Array1<i64>,
    pub(crate) f0s: Array1<f32>,
    pub(crate) volumes: Array1<f32>,
}

impl From<&'_ FrameAudioQuery> for SfDecoderFeature {
    fn from(frame_audio_query: &'_ FrameAudioQuery) -> Self {
        SfDecoderFeature {
            frame_phonemes: frame_audio_query
                .phonemes
                .iter()
                .flat_map(
                    |FramePhoneme {
                         phoneme,
                         frame_length,
                         ..
                     }| {
                        iter::repeat_n(
                            PhonemeCode::from(phoneme.clone()) as _,
                            typeshare::usize_from_u53_saturated(*frame_length),
                        )
                    },
                )
                .collect(),
            // TODO: typed_floatsにissueかPRを出しに行き、スライス変換かbytemuck対応を入れてもらう
            f0s: frame_audio_query
                .f0
                .iter()
                .copied()
                .map(Into::into)
                .collect(),
            volumes: frame_audio_query
                .volume
                .iter()
                .copied()
                .map(Into::into)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::collections::NonEmptySlice;

    #[rstest]
    #[case(&[0, 0, 0], &[0, 0, 0], &[0, 0, 0])]
    #[case(&[0, 0, 30], &[0, 10], &[0, 30])]
    #[case(&[0, 10, 30], &[0, 10], &[10, 30])]
    #[case(&[5, 4, 30], &[0, 10], &[9, 30])]
    fn phoneme_lengths_works(
        #[case] expected: &[u64],
        #[case] consonant_lengths: &[i64],
        #[case] note_durations: &[u32],
    ) {
        assert_eq!(expected, phoneme_lengths(consonant_lengths, note_durations));
    }

    #[test]
    #[should_panic(expected = "must be same length")]
    fn phoneme_lengths_panics_for_jagged() {
        phoneme_lengths(&[0], &[0, 0]);
    }

    #[test]
    #[should_panic(expected = "`consonant_lengths[0]` cannot be non-zero")]
    fn phoneme_lengths_panics_for_non_zero_first_consonant_length() {
        phoneme_lengths(&[1], &[0]);
    }

    fn phoneme_lengths(consonant_lengths: &[i64], note_durations: &[u32]) -> Vec<u64> {
        let consonant_lengths = NonEmptySlice::new(consonant_lengths).unwrap();
        let note_durations = &note_durations
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<_>>();
        let note_durations = NonEmptySlice::new(note_durations).unwrap();
        super::phoneme_lengths(consonant_lengths, note_durations)
            .into_iter()
            .map(Into::into)
            .collect()
    }
}
