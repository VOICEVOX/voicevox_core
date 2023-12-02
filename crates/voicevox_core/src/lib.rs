//! 無料で使える中品質なテキスト読み上げソフトウェア、VOICEVOXのコア。

mod devices;
/// cbindgen:ignore
mod engine;
mod error;
mod infer;
mod macros;
mod manifest;
mod metas;
mod numerics;
mod result;
mod synthesizer;
mod task;
mod user_dict;
mod version;
mod voice_model;

pub mod __internal;
pub mod blocking;
pub mod tokio;

#[cfg(test)]
mod test_util;

#[cfg(test)]
use self::test_util::*;

pub use self::engine::{AccentPhraseModel, AudioQueryModel, TextAnalyzer};
pub use self::error::*;
pub use self::metas::*;
pub use self::result::*;
pub use self::voice_model::*;
pub use devices::*;
pub use manifest::*;
pub use synthesizer::{AccelerationMode, InitializeOptions, SynthesisOptions, TtsOptions};
pub use user_dict::*;
pub use version::*;

use derive_getters::*;
use derive_new::new;
use nanoid::nanoid;
#[cfg(test)]
use rstest::*;

use cfg_if::cfg_if;
