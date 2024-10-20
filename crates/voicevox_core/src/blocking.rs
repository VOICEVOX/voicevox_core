//! ブロッキング版API。

pub use crate::{
    engine::open_jtalk::blocking::OpenJtalk, infer::runtimes::onnxruntime::blocking::Onnxruntime,
    synthesizer::blocking::AudioFeature, synthesizer::blocking::Synthesizer,
    user_dict::dict::blocking::UserDict, voice_model::blocking::VoiceModelFile,
};

pub mod onnxruntime {
    #[cfg(feature = "load-onnxruntime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
    pub use crate::infer::runtimes::onnxruntime::blocking::LoadOnce;
}
