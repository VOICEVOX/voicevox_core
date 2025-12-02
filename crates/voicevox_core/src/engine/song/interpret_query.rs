use std::iter;

use ndarray::Array1;

use crate::{FrameAudioQuery, FramePhoneme};

use super::super::PhonemeCode;

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
