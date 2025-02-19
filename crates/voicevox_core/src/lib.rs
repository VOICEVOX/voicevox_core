//! 無料で使える中品質なテキスト読み上げソフトウェア、VOICEVOXのコア。
//!
//! # Feature flags
//!
//! このクレートの利用にあたっては以下の二つの[Cargoフィーチャ]のうちどちらかを有効にしなければならない。両方の有効化はコンパイルエラーとなる。[`Onnxruntime`]の初期化方法はこれらのフィーチャによって決まる。
//!
//! - **`load-onnxruntime`**: ONNX Runtimeを`dlopen`/`LoadLibraryExW`で開く。[CUDA]と[DirectML]が利用可能。
//! - **`link-onnxruntime`**: ONNX Runtimeをロード時動的リンクする。iOSのような`dlopen`の利用が困難な環境でのみこちらを利用するべきである。_Note_:
//!     [動的リンク対象のライブラリ名]は`onnxruntime`で固定。変更は`patchelf(1)`や`install_name_tool(1)`で行うこと。また、[ONNX RuntimeのGPU機能]を使うことは不可。
//!
//! [Cargoフィーチャ]: https://doc.rust-lang.org/stable/cargo/reference/features.html
//! [CUDA]: https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html
//! [DirectML]: https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html
//! [動的リンク対象のライブラリ名]:
//! https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-lib
//! [`Onnxruntime`]: blocking::Onnxruntime
//! [ONNX RuntimeのGPU機能]: https://onnxruntime.ai/docs/execution-providers/
//!
//! # Example
//!
//! ```
//! use std::io::Cursor;
//!
//! use anyhow::Context as _;
//! use const_format::concatcp;
//!
//! use voicevox_core::{
//!     blocking::{Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile},
//!     CharacterMeta, StyleMeta,
//! };
//!
//! // ダウンローダーにて`onnxruntime`としてダウンロードできるもの
//! # #[cfg(any())]
//! const VVORT: &str = concatcp!(
//!     "./voicevox_core/onnxruntime/lib/",
//!     Onnxruntime::LIB_VERSIONED_FILENAME,
//! );
//! # use test_util::ONNXRUNTIME_DYLIB_PATH as VVORT;
//!
//! // ダウンローダーにて`dict`としてダウンロードできるもの
//! # #[cfg(any())]
//! const OJT_DIC: &str = "./voicevox_core/dict/open_jtalk_dic_utf_8-1.11";
//! # use test_util::OPEN_JTALK_DIC_DIR as OJT_DIC;
//!
//! // ダウンローダーにて`models`としてダウンロードできるもの
//! # #[cfg(any())]
//! const VVM: &str = "./voicevox_core/models/vvms/0.vvm";
//! # use test_util::SAMPLE_VOICE_MODEL_FILE_PATH as VVM;
//!
//! # #[cfg(any())]
//! const TARGET_CHARACTER_NAME: &str = "ずんだもん";
//! # const TARGET_CHARACTER_NAME: &str = "dummy1";
//! #
//! # #[cfg(any())]
//! const TARGET_STYLE_NAME: &str = "ノーマル";
//! # const TARGET_STYLE_NAME: &str = "style1";
//! #
//! const TEXT: &str = "こんにちは";
//!
//! let synth = {
//!     let ort = Onnxruntime::load_once().filename(VVORT).perform()?;
//!     let ojt = OpenJtalk::new(OJT_DIC)?;
//!     Synthesizer::builder(ort).text_analyzer(ojt).build()?
//! };
//!
//! dbg!(synth.is_gpu_mode());
//!
//! synth.load_voice_model(&VoiceModelFile::open(VVM)?)?;
//!
//! let StyleMeta { id: style_id, .. } = synth
//!     .metas()
//!     .into_iter()
//!     .filter(|CharacterMeta { name, .. }| name == TARGET_CHARACTER_NAME)
//!     .flat_map(|CharacterMeta { styles, .. }| styles)
//!     .find(|StyleMeta { name, .. }| name == TARGET_STYLE_NAME)
//!     .with_context(|| {
//!         format!("could not find \"{TARGET_CHARACTER_NAME} ({TARGET_STYLE_NAME})\"")
//!     })?;
//!
//! eprintln!("Synthesizing");
//! let wav = synth.tts(TEXT, style_id).perform()?;
//!
//! eprintln!("Playing the WAV");
//! # if false {
//! play(wav)?;
//! # }
//!
//! fn play(wav: Vec<u8>) -> anyhow::Result<()> {
//!     let (_stream, hdl) = rodio::OutputStream::try_default()?;
//!     let sink = rodio::Sink::try_new(&hdl)?;
//!     sink.append(rodio::Decoder::new_wav(Cursor::new(wav))?);
//!     sink.sleep_until_end();
//!     Ok(())
//! }
//! # Ok::<_, anyhow::Error>(())
//! ```

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

/// ```compile_fail
/// use voicevox_core::__doc;
/// ```
#[cfg(doc)]
#[cfg_attr(docsrs, doc(cfg(doc)))]
pub mod __doc {
    /// [C API]にある以下のアイテムは、Rust API、つまりこのクレートには存在しない。
    ///
    /// | | 理由 |
    /// | :- | :- |
    /// | `VoicevoxLoadOnnxruntimeOptions` | ビルダースタイルであるため |
    /// | `VoicevoxInitializeOptions` | 〃 |
    /// | `VoicevoxSynthesisOptions` | 〃 |
    /// | `VoicevoxTtsOptions` | 〃 |
    /// | `voicevox_make_default_load_onnxruntime_options` | 〃 |
    /// | `voicevox_make_default_initialize_options` | 〃 |
    /// | `voicevox_make_default_synthesis_options` | 〃 |
    /// | `voicevox_make_default_tts_options` | 〃 |
    /// | `voicevox_json_free` | [Rustのデストラクタ機構]があるため |
    /// | `voicevox_wav_free` | 〃 |
    /// | `voicevox_open_jtalk_rc_delete` | 〃 |
    /// | `voicevox_synthesizer_delete` | 〃 |
    /// | `voicevox_voice_model_file_delete` | 〃 |
    /// | `voicevox_user_dict_delete` | 〃 |
    /// | `voicevox_error_result_to_message` | [`std::error::Error`]としてのエラー表示があるため |
    ///
    /// [C API]: https://voicevox.github.io/voicevox_core/apis/c_api/voicevox__core_8h.html
    /// [Rustのデストラクタ機構]: https://doc.rust-lang.org/reference/destructors.html
    #[doc(alias(
        "VoicevoxLoadOnnxruntimeOptions",
        "VoicevoxInitializeOptions",
        "VoicevoxSynthesisOptions",
        "VoicevoxTtsOptions",
        "voicevox_make_default_load_onnxruntime_options",
        "voicevox_make_default_initialize_options",
        "voicevox_make_default_synthesis_options",
        "voicevox_make_default_tts_options",
        "voicevox_json_free",
        "voicevox_wav_free",
        "voicevox_open_jtalk_rc_delete",
        "voicevox_synthesizer_delete",
        "voicevox_voice_model_file_delete",
        "voicevox_user_dict_delete",
        "voicevox_error_result_to_message"
    ))]
    pub mod C_APIには存在するがRust_APIには存在しないアイテム {}
}

mod asyncs;
mod convert;
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
    engine::{AccentPhrase, AudioQuery, Mora},
    error::{Error, ErrorKind},
    metas::{CharacterMeta, CharacterVersion, StyleId, StyleMeta, StyleType, VoiceModelMeta},
    result::Result,
    synthesizer::AccelerationMode,
    user_dict::{UserDictWord, UserDictWordType},
    version::VERSION,
    voice_model::VoiceModelId,
};

// TODO: 後で復活させる
// https://github.com/VOICEVOX/voicevox_core/issues/970
#[doc(hidden)]
pub use self::engine::wav_from_s16le as __wav_from_s16le;
