//! ブロッキング版API。

pub use crate::{
    core::{
        infer::runtimes::onnxruntime::blocking::Onnxruntime, voice_model::blocking::VoiceModelFile,
    },
    engine::talk::{
        open_jtalk::blocking::OpenJtalk, text_analyzer::blocking::TextAnalyzer,
        user_dict::dict::blocking::UserDict,
    },
    synthesizer::blocking::{
        AudioFeature, Synthesizer,
    },
};

pub mod onnxruntime {
    #[cfg(feature = "load-onnxruntime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "load-onnxruntime")))]
    pub use crate::core::infer::runtimes::onnxruntime::blocking::LoadOnce;
}

pub mod synthesizer {
    pub use crate::synthesizer::blocking::{
        Builder, FrameSynthesis, LoadVoiceModel, Synthesis, Tts, TtsFromKana,
    };

    pub use crate::synthesizer::blocking::CreateAudioFeature;
}
