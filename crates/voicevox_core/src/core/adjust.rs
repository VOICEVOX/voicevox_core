//! 推論操作の前処理と後処理。

mod post;
mod pre;

pub(crate) use self::{
    post::{ensure_minimum_phoneme_length, Array1ExtForPostProcess, ArrayExt},
    pre::{pad_decoder_feature, Array1ExtForPreProcess},
};
