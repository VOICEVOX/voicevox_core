use std::iter;

use ndarray::Array1;

use crate::{FrameAudioQuery, FramePhoneme};

use super::{
    super::PhonemeCode,
    frame_audio_query::{PauOrKeyAndLyric, ValidatedNote},
};

pub(crate) fn repeat_phoneme_code_and_key(
    frame_phoneme: &FramePhoneme,
    note: &ValidatedNote,
) -> impl Iterator<Item = (i64, i64)> {
    let phoneme = PhonemeCode::from(frame_phoneme.phoneme.clone()) as _;
    let key = note.pau_or_key_and_lyric.key();
    let n = typeshare::usize_from_u53_saturated(frame_phoneme.frame_length);
    iter::repeat_n((phoneme, key), n)
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
