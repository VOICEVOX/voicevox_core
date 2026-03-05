//! 推論操作の前処理と後処理。

mod post;
mod pre;

pub(crate) use self::{
    post::{Array1ExtForPostProcess, ArrayExt, ensure_minimum_phoneme_length},
    pre::{Array1ExtForPreProcess, pad_decoder_feature},
};
