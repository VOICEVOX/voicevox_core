//! ブロッキング版API。

pub use crate::{
    engine::open_jtalk::blocking::OpenJtalk, synthesizer::blocking::Synthesizer,
    user_dict::dict::blocking::UserDict, voice_model::blocking::VoiceModel,
};
