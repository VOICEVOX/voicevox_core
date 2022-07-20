#![deny(unsafe_code)]

#[allow(unsafe_code)]
mod c_export;
/// cbindgen:ignore
mod engine;
mod error;
mod internal;
mod result;
mod status;

#[cfg(test)]
mod test_util;

#[cfg(test)]
use test_util::*;

use error::*;
use result::*;

use derive_getters::*;
use derive_new::new;
#[cfg(test)]
use rstest::*;

use cfg_if::cfg_if;
