use std::iter;

use ndarray::Array1;

use crate::{FrameAudioQuery, FramePhoneme};

use super::{
    super::PhonemeCode,
    frame_audio_query::{PauOrKeyAndLyric, ValidatedNote},
};

#[derive(Clone, Copy)]
pub(crate) struct FramePhonemeWithKey {
    phoneme: PhonemeCode,
    frame_length: usize,
    key: i64,
}

impl FramePhonemeWithKey {
    pub(crate) fn new(frame_phoneme: &FramePhoneme, note: &ValidatedNote) -> Self {
        Self {
            phoneme: frame_phoneme.phoneme.clone().into(),
            frame_length: typeshare::usize_from_u53_saturated(frame_phoneme.frame_length),
            key: note.pau_or_key_and_lyric.key(),
        }
    }

    pub(crate) fn repeat_phoneme(self) -> impl Iterator<Item = i64> {
        iter::repeat_n(self.phoneme as _, self.frame_length)
    }

    pub(crate) fn repeat_key(self) -> impl Iterator<Item = i64> {
        iter::repeat_n(self.key, self.frame_length)
    }
}

impl PauOrKeyAndLyric {
    fn key(&self) -> i64 {
        match *self {
            Self::Pau => -1,
            Self::KeyAndLyric { key, .. } => u64::from(key) as _,
        }
    }
}

pub(crate) struct SfDecoderFeature {
    pub(crate) frame_phonemes: Array1<i64>,
    pub(crate) f0s: Array1<f32>,
    pub(crate) volumes: Array1<f32>,
}

impl FrameAudioQuery {
    pub(crate) fn sf_decoder_feature(&self) -> SfDecoderFeature {
        SfDecoderFeature {
            frame_phonemes: self
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
            f0s: self.f0.iter().copied().map(Into::into).collect(),
            volumes: self.volume.iter().copied().map(Into::into).collect(),
        }
    }
}
