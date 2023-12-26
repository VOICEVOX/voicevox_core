//! Tokio版API。

pub use crate::{
    engine::open_jtalk::tokio::OpenJtalk, synthesizer::tokio::Synthesizer,
    user_dict::dict::tokio::UserDict, voice_model::tokio::VoiceModel,
};
