#![deny(unsafe_code)]

mod devices;
/// cbindgen:ignore
mod engine;
mod error;
mod inference_core;
mod macros;
mod manifest;
mod metas;
mod numerics;
mod result;
pub mod result_code;
mod status;
mod user_dict;
mod version;
mod voice_model;
mod voice_synthesizer;

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
pub use version::*;
pub use voice_synthesizer::*;
pub use user_dict::*;

use derive_getters::*;
use derive_new::new;
use nanoid::nanoid;
#[cfg(test)]
use rstest::*;

use cfg_if::cfg_if;
