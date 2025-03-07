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
//! # 音声の調整
//!
//! ユーザーガイドの[テキスト音声合成の流れ]を参照。
//!
//! 以下の`wav1`から`wav4`はすべて同一となる。
//!
//! [テキスト音声合成の流れ]: https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/tts-process.md
//!
//! ```
//! use std::collections::HashSet;
//!
//! use voicevox_core::{
//!     blocking::{Synthesizer, TextAnalyzer},
//!     AudioQuery, StyleId,
//! };
//! #
//! # use test_util::{ONNXRUNTIME_DYLIB_PATH, OPEN_JTALK_DIC_DIR, SAMPLE_VOICE_MODEL_FILE_PATH};
//! # use voicevox_core::blocking::{Onnxruntime, OpenJtalk, VoiceModelFile};
//!
//! fn f(synth: &Synthesizer<impl TextAnalyzer>) -> anyhow::Result<()> {
//! #    const TEXT: &str = "";
//! #   #[cfg(any())]
//!     const TEXT: &str = _;
//! #
//! #   const STYLE_ID: StyleId = StyleId(0);
//! #   #[cfg(any())]
//!     const STYLE_ID: StyleId = _;
//!
//!     let wav1 = synth.tts(TEXT, STYLE_ID).perform()?;
//!
//!     let wav2 = {
//!         let query = synth.create_audio_query(TEXT, STYLE_ID)?;
//!         synth.synthesis(&query, STYLE_ID).perform()?
//!     };
//!
//!     let wav3 = {
//!         let phrases = synth.create_accent_phrases(TEXT, STYLE_ID)?;
//!         let query = AudioQuery::from(phrases);
//!         synth.synthesis(&query, STYLE_ID).perform()?
//!     };
//!
//!     let wav4 = {
//!         let phrases = synth.text_analyzer().analyze(TEXT)?;
//!         let phrases = synth.replace_mora_data(&phrases, STYLE_ID)?;
//!         let query = AudioQuery::from(phrases);
//!         synth.synthesis(&query, STYLE_ID).perform()?
//!     };
//!
//!     let wav5 = {
//!         let phrases = synth.text_analyzer().analyze(TEXT)?;
//!         let phrases = synth.replace_phoneme_length(&phrases, STYLE_ID)?;
//!         let phrases = synth.replace_mora_pitch(&phrases, STYLE_ID)?;
//!         let query = AudioQuery::from(phrases);
//!         synth.synthesis(&query, STYLE_ID).perform()?
//!     };
//!
//!     assert_eq!(1, HashSet::from([wav1, wav2, wav3, wav4, wav5]).len());
//!     Ok(())
//! }
//! #
//! # let synth = &{
//! #     let ort = Onnxruntime::load_once()
//! #         .filename(ONNXRUNTIME_DYLIB_PATH)
//! #         .perform()?;
//! #     let ojt = OpenJtalk::new(OPEN_JTALK_DIC_DIR)?;
//! #     Synthesizer::builder(ort).text_analyzer(ojt).build()?
//! # };
//! # synth.load_voice_model(&VoiceModelFile::open(SAMPLE_VOICE_MODEL_FILE_PATH)?)?;
//! # f(synth)?;
//! # anyhow::Ok(())
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
