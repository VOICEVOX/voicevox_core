//! 無料で使える中品質なテキスト読み上げソフトウェア、VOICEVOXのコア。
//!
//! # Feature flags
//!
//! このクレートの利用にあたっては以下の二つの[Cargoフィーチャ]のうちどちらかを有効にしなければなり
//! ません。両方の有効化はコンパイルエラーとなります。[`Onnxruntime`]の初期化方法はこれらの
//! フィーチャによって決まります。
//!
//! - **`load-onnxruntime`**: ONNX Runtimeを`dlopen`/`LoadLibraryExW`で
//!     開きます。[CUDA]と[DirectML]が利用できます。
//! - **`link-onnxruntime`**: ONNX Runtimeをロード時動的リンクします。iOSのような`dlopen`の利用が
//!     困難な環境でのみこちらを利用するべきです。_Note_:
//!     [動的リンク対象のライブラリ名]は`onnxruntime`で固定です。変更
//!     は`patchelf(1)`や`install_name_tool(1)`で行ってください。また、[ONNX RuntimeのGPU機能]を使う
//!     ことはできません。
//!
//! [Cargoフィーチャ]: https://doc.rust-lang.org/stable/cargo/reference/features.html
//! [CUDA]: https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html
//! [DirectML]: https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html
//! [動的リンク対象のライブラリ名]:
//! https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-lib
//! [`Onnxruntime`]: blocking::Onnxruntime
//! [ONNX RuntimeのGPU機能]: https://onnxruntime.ai/docs/execution-providers/

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

mod asyncs;
mod core;
mod devices;
/// cbindgen:ignore
mod engine;
mod error;
mod future;
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

#[doc(hidden)]
pub mod __internal;
pub mod blocking;
pub mod nonblocking;

#[cfg(test)]
mod test_util;

#[expect(
    clippy::single_component_path_imports,
    reason = "https://crates.io/crates/rstest_reuse/0.6.0#use-rstest_resuse-at-the-top-of-your-crate"
)]
#[cfg(test)]
use rstest_reuse;

pub use self::{
    devices::SupportedDevices,
    engine::{wav_from_s16le, AccentPhrase, AudioQuery, Mora},
    error::{Error, ErrorKind},
    metas::{CharacterMeta, CharacterVersion, StyleId, StyleMeta, StyleType, VoiceModelMeta},
    result::Result,
    synthesizer::AccelerationMode,
    user_dict::{UserDictWord, UserDictWordType},
    version::VERSION,
    voice_model::VoiceModelId,
};
