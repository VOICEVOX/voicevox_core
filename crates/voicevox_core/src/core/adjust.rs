//! 推論操作の前処理と後処理。

mod post;
mod pre;

pub(crate) use self::{
    post::{ensure_minimum_phoneme_length, ArrayExt},
    pre::pad_decoder_feature,
};
