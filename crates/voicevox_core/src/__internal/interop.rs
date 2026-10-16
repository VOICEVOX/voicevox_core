pub mod raii;

pub use crate::{
    convert::ToJsonValue,
    core::adjust::MARGIN,
    core::metas::merge as merge_metas,
    engine::frame::WAVE_SAMPLES_PER_FRAME,
    engine::talk::{
        DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
        user_dict::{DEFAULT_WORD_TYPE, InvalidWordError},
    },
    engine::validate::Validate,
    synthesizer::{
        BlockingTextAnalyzerExt, DEFAULT_CPU_NUM_THREADS, DEFAULT_HEAVY_INFERENCE_CANCELLABLE,
        NonblockingTextAnalyzerExt, blocking::PerformInference,
    },
};

pub trait PeekIfEmpty {
    fn is_empty(&self) -> bool;
}
