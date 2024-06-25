//! ブロッキング版API。

pub use crate::{
    engine::open_jtalk::blocking::OpenJtalk, infer::runtimes::onnxruntime::blocking::Onnxruntime,
    synthesizer::blocking::Synthesizer, user_dict::dict::blocking::UserDict,
    voice_model::blocking::VoiceModel,
};

pub mod onnxruntime {
    #[cfg(feature = "onnxruntime-libloading")]
    #[cfg_attr(docsrs, doc(cfg(feature = "onnxruntime-libloading")))]
    pub use crate::infer::runtimes::onnxruntime::blocking::LoadOnce;
}
