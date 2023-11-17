//! 無料で使える中品質なテキスト読み上げソフトウェア、VOICEVOXのコア。

mod devices;
/// cbindgen:ignore
mod engine;
mod error;
mod infer;
mod inference_core;
mod macros;
mod manifest;
mod metas;
mod numerics;
mod result;
mod synthesizer;
mod user_dict;
mod version;
mod voice_model;

#[doc(hidden)]
pub mod __internal;

use self::inference_core::*;

#[cfg(test)]
mod test_util;

#[cfg(test)]
use self::test_util::*;

pub use self::engine::{AccentPhraseModel, AudioQueryModel, OpenJtalk};
pub use self::error::*;
pub use self::metas::*;
pub use self::result::*;
pub use self::voice_model::*;
pub use devices::*;
pub use manifest::*;
pub use synthesizer::*;
pub use user_dict::*;
pub use version::*;

use derive_getters::*;
use derive_new::new;
use nanoid::nanoid;
#[cfg(test)]
use rstest::*;

use cfg_if::cfg_if;
