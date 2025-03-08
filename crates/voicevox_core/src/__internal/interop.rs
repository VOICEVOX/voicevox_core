pub mod raii;

pub use crate::{
    convert::ToJsonValue,
    metas::merge as merge_metas,
    synthesizer::{
        blocking::PerformInference, BlockingTextAnalyzerExt, NonblockingTextAnalyzerExt,
        DEFAULT_CPU_NUM_THREADS, DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK,
        DEFAULT_HEAVY_INFERENCE_CANCELLABLE, MARGIN,
    },
    user_dict::{DEFAULT_PRIORITY, DEFAULT_WORD_TYPE},
};
