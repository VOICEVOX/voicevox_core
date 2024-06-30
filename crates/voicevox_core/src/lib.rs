//! 無料で使える中品質なテキスト読み上げソフトウェア、VOICEVOXのコア。

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(any(feature = "load-onnxruntime", feature = "link-onnxruntime")))]
compile_error!("either `load-onnxruntime` or `link-onnxruntime` must be enabled");

#[cfg(not(doc))]
const _: () = {
    #[cfg(all(feature = "load-onnxruntime", feature = "link-onnxruntime"))]
    compile_error!("`load-onnxruntime` and `link-onnxruntime` cannot be enabled at the same time");

    // Rust APIでvoicevox-ortを他のクレートが利用する可能性を考え、voicevox-ort側とfeatureがズレ
    // ないようにする

    #[cfg(feature = "load-onnxruntime")]
    ort::assert_feature!(
        cfg(feature = "load-dynamic"),
        "when `load-onnxruntime` is enabled,`voicevox-ort/load-dynamic` must be also enabled",
    );

    #[cfg(feature = "link-onnxruntime")]
    ort::assert_feature!(
        cfg(not(feature = "load-dynamic")),
        "when `link-onnxruntime` is enabled,`voicevox-ort/load-dynamic` must be disabled",
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
