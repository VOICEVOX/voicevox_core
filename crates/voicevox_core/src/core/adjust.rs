//! 推論操作の前処理と後処理。

mod post;
mod pre;

pub(crate) use self::{
    post::{
        Array1ExtForPostProcess, ArrayExt, ensure_minimum_phoneme_length, ensure_non_nan_finite,
        ensure_positive_finite, trim_margin_from_wave,
    },
    pre::{Array1ExtForPreProcess, PADDING_FRAME_LENGTH, pad_decoder_feature},
};

pub use self::post::MARGIN;
