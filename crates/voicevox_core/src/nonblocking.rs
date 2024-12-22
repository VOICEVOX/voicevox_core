//! 非同期版API。
//!
//! # Performance
//!
//! これらは[blocking]クレートにより動いている。特定の非同期ランタイムを必要とせず、[pollster]など
//! でも動かすことができる。
//!
//! スレッドプールおよびエグゼキュータはblockingクレートに依存するすべてのプログラム間で共有される。
//! スレッドプールのサイズは、blockingクレートの説明にある通り`$BLOCKING_MAX_THREADS`で調整すること
//! ができる。
//!
//! また未調査ではあるが、[`cpu_num_threads`]を`0`にするのは適切ではない可能性がある ([VOICEVOX/voicevox_core#902])。
//!
//! [blocking]: https://docs.rs/crate/blocking
//! [pollster]: https://docs.rs/crate/pollster
//! [VOICEVOX/voicevox_core#902]: https://github.com/VOICEVOX/voicevox_core/issues/902
//! [`cpu_num_threads`]: crate::InitializeOptions::cpu_num_threads

pub use crate::{
    engine::open_jtalk::nonblocking::OpenJtalk,
    infer::runtimes::onnxruntime::nonblocking::Onnxruntime, synthesizer::nonblocking::Synthesizer,
    user_dict::dict::nonblocking::UserDict, voice_model::nonblocking::VoiceModelFile,
};

pub mod onnxruntime {
    #[cfg(feature = "load-onnxruntime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
    pub use crate::infer::runtimes::onnxruntime::nonblocking::LoadOnce;
}
