pub mod raii;

pub use crate::{
    convert::ToJsonValue,
    metas::merge as merge_metas,
    synthesizer::{
        blocking::PerformInference, DEFAULT_ASYNC_CANCELLABLE, DEFAULT_CPU_NUM_THREADS,
        DEFAULT_ENABLE_INTERROGATIVE_UPSPEAK, MARGIN,
    },
    user_dict::{DEFAULT_PRIORITY, DEFAULT_WORD_TYPE},
};
