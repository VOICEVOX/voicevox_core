use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// ユーザー辞書の単語。
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
pub struct UserDictWord {
    surface: String,
    pronunciation: String,
    accent_type: i32,
    word_type: UserDictWordType,
    priority: i32,
}

/// ユーザー辞書の単語の種類。
#[derive(Clone, Debug, PartialEq)]
pub enum UserDictWordType {
    /// 固有名詞。
    ProperNoun,
    /// 一般名詞。
    CommonNoun,
    /// 動詞。
    Verb,
    /// 形容詞。
    Adjective,
    /// 接尾辞。
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[rstest]
    #[case(crate::UserDictWordType::ProperNoun, "PROPER_NOUN")]
    #[case(crate::UserDictWordType::CommonNoun, "COMMON_NOUN")]
    #[case(crate::UserDictWordType::Verb, "VERB")]
    #[case(crate::UserDictWordType::Adjective, "ADJECTIVE")]
    #[case(crate::UserDictWordType::Suffix, "SUFFIX")]
    fn serialize_works(#[case] word_type: super::UserDictWordType, #[case] expected: &str) {
        let serialized = serde_json::to_string(&word_type).unwrap();
        assert_eq!(serialized, format!("\"{}\"", expected));
    }

    #[rstest]
    #[case("PROPER_NOUN", crate::UserDictWordType::ProperNoun)]
    #[case("COMMON_NOUN", crate::UserDictWordType::CommonNoun)]
    #[case("VERB", crate::UserDictWordType::Verb)]
    #[case("ADJECTIVE", crate::UserDictWordType::Adjective)]
    #[case("SUFFIX", crate::UserDictWordType::Suffix)]
    fn deserialize_works(#[case] serialized: &str, #[case] expected: super::UserDictWordType) {
        let word_type: super::UserDictWordType =
            serde_json::from_str(format!("\"{}\"", serialized).as_str()).unwrap();
        assert_eq!(word_type, expected);
    }
}
