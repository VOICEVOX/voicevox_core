//! ブロッキング版API。

pub use crate::{
    engine::open_jtalk::blocking::OpenJtalk, infer::runtimes::onnxruntime::blocking::Onnxruntime,
    synthesizer::blocking::Synthesizer, text_analyzer::blocking::TextAnalyzer,
    user_dict::dict::blocking::UserDict, voice_model::blocking::VoiceModelFile,
};

// TODO: 後で復活させる
// https://github.com/VOICEVOX/voicevox_core/issues/970
#[doc(hidden)]
pub use crate::synthesizer::blocking::AudioFeature as __AudioFeature;

pub mod onnxruntime {
    #[cfg(feature = "load-onnxruntime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
    pub use crate::infer::runtimes::onnxruntime::blocking::LoadOnce;
}

pub mod synthesizer {
    pub use crate::synthesizer::blocking::{Builder, Synthesis, Tts, TtsFromKana};

    // TODO: 後で復活させる
    // https://github.com/VOICEVOX/voicevox_core/issues/970
    //pub use crate::synthesizer::blocking::PrecomputeRender;
}
