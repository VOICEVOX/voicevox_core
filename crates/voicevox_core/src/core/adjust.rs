//! 推論操作の前処理と後処理。

mod post;
mod pre;

pub(crate) use self::{
    post::{
        Array1ExtForPostProcess, ArrayExt, ensure_minimum_phoneme_length, ensure_non_nan_finite,
        ensure_positive_finite,
    },
    pre::{Array1ExtForPreProcess, pad_decoder_feature},
};
