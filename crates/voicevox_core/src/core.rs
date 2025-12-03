//! 音声モデルの取り扱いと推論に関する「コア」の領域。

pub(crate) mod adjust;
pub(crate) mod devices;
pub(crate) mod infer;
mod manifest;
pub(crate) mod metas;
pub(crate) mod status;
pub(crate) mod voice_model;

pub(crate) use self::adjust::{ensure_minimum_phoneme_length, pad_decoder_feature, Array2Ext};
