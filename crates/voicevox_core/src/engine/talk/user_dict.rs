pub(crate) mod dict;
mod part_of_speech_data;
mod word;

pub(crate) use self::word::{validate_pronunciation, InvalidWordError};
pub use self::word::{
    UserDictWord, UserDictWordBuilder, UserDictWordType, DEFAULT_PRIORITY, DEFAULT_WORD_TYPE,
};
