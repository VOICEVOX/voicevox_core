#![deny(clippy::all)]

pub(crate) fn convert_result<T>(result: voicevox_core::Result<T>) -> napi::Result<T> {
    result.map_err(|err| napi::Error::from_reason(err.to_string()))
}

pub mod devices;
pub mod model;
pub mod namespaces;
pub mod synthesizer;
pub mod word;

#[macro_use]
extern crate napi_derive;
