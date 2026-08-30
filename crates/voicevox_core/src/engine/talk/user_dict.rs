pub(crate) mod dict;
mod part_of_speech_data;
mod word;

pub use self::word::{
    DEFAULT_WORD_TYPE, InvalidWordError, UserDictWord, UserDictWordBuilder, UserDictWordPriority,
    UserDictWordType,
};

pub(crate) use self::word::validate_pronunciation;
