//! ブロッキング版API。

pub use crate::{
    core::{
        infer::runtimes::onnxruntime::blocking::Onnxruntime, voice_model::blocking::VoiceModelFile,
    },
    engine::talk::{
        open_jtalk::blocking::OpenJtalk, text_analyzer::blocking::TextAnalyzer,
        user_dict::dict::blocking::UserDict,
    },
    synthesizer::blocking::Synthesizer,
};

// TODO: 後で復活させる
// https://github.com/VOICEVOX/voicevox_core/issues/970
#[doc(hidden)]
pub use crate::synthesizer::blocking::AudioFeature as __AudioFeature;

pub mod onnxruntime {
    #[cfg(feature = "load-onnxruntime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
    pub use crate::core::infer::runtimes::onnxruntime::blocking::LoadOnce;
}

pub mod synthesizer {
    pub use crate::synthesizer::blocking::{Builder, FrameSynthesis, Synthesis, Tts, TtsFromKana};

    // TODO: 後で復活させる
    // https://github.com/VOICEVOX/voicevox_core/issues/970
    //pub use crate::synthesizer::blocking::PrecomputeRender;
}
