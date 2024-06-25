//! Tokio版API。

pub use crate::{
    engine::open_jtalk::tokio::OpenJtalk, infer::runtimes::onnxruntime::tokio::Onnxruntime,
    synthesizer::tokio::Synthesizer, user_dict::dict::tokio::UserDict,
    voice_model::tokio::VoiceModel,
};

pub mod onnxruntime {
    #[cfg(feature = "onnxruntime-libloading")]
    #[cfg_attr(docsrs, doc(cfg(feature = "onnxruntime-libloading")))]
    pub use crate::infer::runtimes::onnxruntime::tokio::LoadOnce;
}
