use derive_getters::Getters;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
pub struct UserDictWord {
    surface: String,
    pronunciation: String,
    accent_type: i32,
    word_type: UserDictWordType,
    priority: i32,
}

#[derive(Clone, Debug)]
pub enum UserDictWordType {
    ProperNoun,
    CommonNoun,
    Verb,
    Adjective,
    Suffix,
}

impl Serialize for UserDictWordType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            UserDictWordType::ProperNoun => serializer.serialize_str("PROPER_NOUN"),
            UserDictWordType::CommonNoun => serializer.serialize_str("COMMON_NOUN"),
            UserDictWordType::Verb => serializer.serialize_str("VERB"),
            UserDictWordType::Adjective => serializer.serialize_str("ADJECTIVE"),
            UserDictWordType::Suffix => serializer.serialize_str("SUFFIX"),
        }
    }
}

impl<'de> Deserialize<'de> for UserDictWordType {
    fn deserialize<D>(deserializer: D) -> Result<UserDictWordType, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "PROPER_NOUN" => Ok(UserDictWordType::ProperNoun),
            "COMMON_NOUN" => Ok(UserDictWordType::CommonNoun),
            "VERB" => Ok(UserDictWordType::Verb),
            "ADJECTIVE" => Ok(UserDictWordType::Adjective),
            "SUFFIX" => Ok(UserDictWordType::Suffix),
            _ => Err(serde::de::Error::custom(format!(
                "invalid UserDictWordType: {}",
                s
            ))),
        }
    }
}
