//! 無料で使える中品質なテキスト読み上げソフトウェア、VOICEVOXのコア。
//!
//! # Feature flags
//!
//! このクレートの利用にあたっては以下の二つの[Cargoフィーチャ]のうちどちらかを有効にしなければならない。両方の有効化はコンパイルエラーとなる。[`Onnxruntime`]の初期化方法はこれらのフィーチャによって決まる。
//!
//! - **`load-onnxruntime`**: ONNX Runtimeを`dlopen`/`LoadLibraryExW`で開く。[CUDA]と[DirectML]が利用可能。
//! - **`link-onnxruntime`**: ONNX Runtimeをロード時動的リンクする。iOSのような`dlopen`の利用が困難な環境でのみこちらを利用するべきである。_Note_:
//!   [動的リンク対象のライブラリ名]は`onnxruntime`で固定。変更は`patchelf(1)`や`install_name_tool(1)`で行うこと。また、[ONNX RuntimeのGPU機能]を使うことは不可。
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
//! use std::{io::Write as _, panic};
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
//! let wav = &synth.tts(TEXT, style_id).perform()?;
//!
//! eprintln!("Playing the WAV");
//! # if false {
//! play(wav)?;
//! # }
//!
//! fn play(wav: &[u8]) -> anyhow::Result<()> {
//!     let tempfile = tempfile::Builder::new().suffix(".wav").tempfile()?;
//!     (&tempfile).write_all(wav)?;
//!     let tempfile = &tempfile.into_temp_path();
//!     open::that_in_background(tempfile)
//!         .join()
//!         .unwrap_or_else(|e| panic::resume_unwind(e))?;
//!     Ok(())
//! }
//! # Ok::<_, anyhow::Error>(())
//! ```
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
/// cbindgen:ignore
mod engine;
mod error;
mod future;
mod macros;
mod result;
mod synthesizer;
mod task;
mod version;

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
    core::{
        devices::SupportedDevices,
        metas::{CharacterMeta, CharacterVersion, StyleId, StyleMeta, StyleType, VoiceModelMeta},
        voice_model::VoiceModelId,
    },
    engine::{
        song::{FrameAudioQuery, FramePhoneme, Note, NoteId, OptionalLyric, Score},
        talk::{
            user_dict::{UserDictWord, UserDictWordBuilder, UserDictWordType},
            AccentPhrase, AudioQuery, Mora,
        },
        Phoneme, SamplingRate, Sil,
    },
    error::{Error, ErrorKind},
    result::Result,
    synthesizer::AccelerationMode,
    version::VERSION,
};

// TODO: 後で復活させる
// https://github.com/VOICEVOX/voicevox_core/issues/970
#[doc(hidden)]
pub use self::engine::wav_from_s16le as __wav_from_s16le;
