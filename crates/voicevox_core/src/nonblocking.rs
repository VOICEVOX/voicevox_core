//! 非同期版API。
//!
//! # Performance
//!
//! これらは[blocking]クレートにより動いている。特定の非同期ランタイムを必要とせず、[pollster]などでも動かすことができる。
//!
//! スレッドプールおよびエグゼキュータはblockingクレートに依存するすべてのプログラム間で共有される。スレッドプールのサイズは、blockingクレートの説明にある通り`$BLOCKING_MAX_THREADS`で調整することができる。
//!
//! また未調査ではあるが、このモジュールについては[`cpu_num_threads`]は物理コアの数+1を指定するのが適切な可能性がある
//! ([VOICEVOX/voicevox_core#902])。
//!
//! [blocking]: https://docs.rs/crate/blocking
//! [pollster]: https://docs.rs/crate/pollster
//! [VOICEVOX/voicevox_core#902]: https://github.com/VOICEVOX/voicevox_core/issues/902
//! [`cpu_num_threads`]: crate::nonblocking::synthesizer::Builder::cpu_num_threads

pub use crate::{
    core::{
        infer::runtimes::onnxruntime::nonblocking::Onnxruntime,
        voice_model::nonblocking::VoiceModelFile,
    },
    engine::talk::{
        open_jtalk::nonblocking::OpenJtalk, text_analyzer::nonblocking::TextAnalyzer,
        user_dict::dict::nonblocking::UserDict,
    },
    synthesizer::nonblocking::Synthesizer,
};

pub mod onnxruntime {
    #[cfg(feature = "load-onnxruntime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
    pub use crate::core::infer::runtimes::onnxruntime::nonblocking::LoadOnce;
}

pub mod synthesizer {
    pub use crate::synthesizer::nonblocking::{
        Builder, FrameSysnthesis, Synthesis, Tts, TtsFromKana,
    };
}
