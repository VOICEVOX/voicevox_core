//! 無料で使える中品質なテキスト読み上げソフトウェア、VOICEVOXのコア。

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(any(feature = "onnxruntime-libloading", feature = "onnxruntime-link-dylib")))]
compile_error!("either `onnxruntime-libloading` or `onnxruntime-link-dylib` must be enabled");

#[cfg(not(doc))]
const _: () = {
    #[cfg(all(feature = "onnxruntime-libloading", feature = "onnxruntime-link-dylib"))]
    compile_error!(
        "`onnxruntime-libloading` and `onnxruntime-link-dylib` cannot be enabled at the same time",
    );

    // Rust APIでvoicevox-ortを他のクレートが利用する可能性を考え、voicevox-ort側とfeatureがズレ
    // ないようにする

    #[cfg(feature = "onnxruntime-libloading")]
    ort::assert_load_dynamic_is_enabled!(
        "when `onnxruntime-libloading` is enabled,`voicevox-ort/load-dynamic` must be also enabled",
    );

    #[cfg(feature = "onnxruntime-link-dylib")]
    ort::assert_load_dynamic_is_disabled!(
        "when `onnxruntime-link-dylib` is enabled,`voicevox-ort/load-dynamic` must be disabled",
    );
};

mod devices;
/// cbindgen:ignore
mod engine;
mod error;
mod infer;
mod macros;
mod manifest;
mod metas;
mod result;
mod status;
mod synthesizer;
mod task;
mod text_analyzer;
mod user_dict;
mod version;
mod voice_model;

pub mod __internal;
pub mod blocking;
pub mod tokio;

#[cfg(test)]
mod test_util;

// https://crates.io/crates/rstest_reuse#use-rstest_resuse-at-the-top-of-your-crate
#[allow(clippy::single_component_path_imports)]
#[cfg(test)]
use rstest_reuse;

pub use self::{
    devices::SupportedDevices,
    engine::{AccentPhraseModel, AudioQueryModel, FullcontextExtractor},
    error::{Error, ErrorKind},
    metas::{
        RawStyleId, RawStyleVersion, SpeakerMeta, StyleId, StyleMeta, StyleType, StyleVersion,
        VoiceModelMeta,
    },
    result::Result,
    synthesizer::{AccelerationMode, InitializeOptions, SynthesisOptions, TtsOptions},
    user_dict::{UserDictWord, UserDictWordType},
    version::VERSION,
    voice_model::{RawVoiceModelId, VoiceModelId},
};
