//! 無料で使える中品質なテキスト読み上げソフトウェア、VOICEVOXのコア。
//!
//! # Feature flags
//!
//! ## ONNX Runtimeのリンク方法を決めるフィーチャ
//!
//! このクレートの利用にあたっては以下の二つの[Cargoフィーチャ]のうちどちらかを有効にしなければなり
//! ません。両方の有効化はコンパイルエラーとなります。[`Onnxruntime`]の初期化方法はこれらの
//! フィーチャによって決まります。
//!
//! - **`load-onnxruntime`**: ONNX Runtimeを`dlopen`/`LoadLibraryExW`で開きます。
//! - **`link-onnxruntime`**: ONNX Runtimeをロード時動的リンクします。iOSのような`dlopen`の利用が
//!     困難な環境でのみこちらを利用するべきです。_Note_:
//!     [動的リンク対象のライブラリ名]は`onnxruntime`で固定です。変更
//!     は`patchelf(1)`や`install_name_tool(1)`で行ってください。
//!
//! ## GPUを利用可能にするフィーチャ
//!
//! - **`cuda`**
//! - **`directml`**
// TODO: こんな感じ(↓)で書く
////! - **`cuda`**: [CUDAを用いた機械学習推論]を可能にします。
////!     - ❗ <code>[acceleration\_mode]={Gpu,Auto}</code>のときの挙動が変化します。`directml`と共に
////!         有効化したときの挙動は未規定です。
////! - **`directml`**: [DirectMLを用いた機械学習推論]を可能にします。
////!     - ❗ 〃
////!
////! [CUDAを用いた機械学習推論]:
////! https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html
////! [DirectMLを用いた機械学習推論]:
////! https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html
////! [acceleration\_mode]: InitializeOptions::acceleration_mode
//!
//! [Cargoフィーチャ]: https://doc.rust-lang.org/stable/cargo/reference/features.html
//! [動的リンク対象のライブラリ名]:
//! https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-lib
//! [`Onnxruntime`]: blocking::Onnxruntime

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
    engine::{AccentPhrase, AudioQuery, FullcontextExtractor, Mora},
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
