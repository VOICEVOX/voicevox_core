//! 無料で使える中品質なテキスト読み上げソフトウェア、VOICEVOXのコア。

mod devices;
/// cbindgen:ignore
mod engine;
mod error;
mod infer;
mod macros;
mod manifest;
mod metas;
mod numerics;
mod result;
mod synthesizer;
mod task;
mod text_analyzer;
mod user_dict;
mod version;
mod voice_model;

pub mod __internal;
pub mod blocking;
pub mod tokio;

#[cfg(test)]
mod test_util;

pub use self::{
    devices::SupportedDevices,
    engine::{AccentPhraseModel, AudioQueryModel, FullcontextExtractor},
    error::{Error, ErrorKind},
    metas::{
        RawStyleId, RawStyleVersion, SpeakerMeta, StyleId, StyleMeta, StyleVersion, VoiceModelMeta,
    },
    result::Result,
    synthesizer::{AccelerationMode, InitializeOptions, SynthesisOptions, TtsOptions},
    user_dict::{UserDictWord, UserDictWordType},
    version::VERSION,
    voice_model::{RawVoiceModelId, VoiceModelId},
};
