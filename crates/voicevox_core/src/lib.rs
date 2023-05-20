#![deny(unsafe_code)]

/// cbindgen:ignore
mod engine;
mod error;
mod macros;
mod numerics;
mod publish;
mod result;
pub mod result_code;
mod status;

pub use self::publish::*;

pub use self::engine::{AccentPhraseModel, AudioQueryModel};
pub use self::error::*;
pub use self::result::*;

use derive_getters::*;
use derive_new::new;
#[cfg(test)]
use rstest::*;

use cfg_if::cfg_if;
