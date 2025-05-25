pub mod raii;

pub use crate::{
    convert::ToJsonValue,
    core::metas::merge as merge_metas,
    engine::talk::user_dict::{DEFAULT_PRIORITY, DEFAULT_WORD_TYPE},
    synthesizer::{
        BlockingTextAnalyzerExt, DEFAULT_CPU_NUM_THREADS, DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
        DEFAULT_HEAVY_INFERENCE_CANCELLABLE, MARGIN, NonblockingTextAnalyzerExt,
        blocking::PerformInference,
    },
};
