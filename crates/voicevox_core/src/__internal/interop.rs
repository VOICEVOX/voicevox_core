pub mod raii;

pub use crate::{
    audio_feature::{AudioFeatureOutOfBoundsError, AudioFeatureSliceExt},
    convert::ToJsonValue,
    core::adjust::MARGIN,
    core::metas::merge as merge_metas,
    dim::AUDIO_FEATURE_BINS,
    engine::talk::{
        DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
        user_dict::{DEFAULT_WORD_TYPE, InvalidWordError},
    },
    engine::validate::Validate,
    frame::WAVE_SAMPLES_PER_FRAME,
    synthesizer::{
        BlockingTextAnalyzerExt, DEFAULT_CPU_NUM_THREADS, DEFAULT_HEAVY_INFERENCE_CANCELLABLE,
        NonblockingTextAnalyzerExt, blocking::PerformInference,
    },
};

pub trait PeekIfEmpty {
    fn is_empty(&self) -> bool;
}
