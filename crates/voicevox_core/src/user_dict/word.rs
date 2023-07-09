use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// ユーザー辞書の単語。
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
pub struct UserDictWord {
    /// 単語の表記。
    pub surface: String,
    /// 単語の読み。
    pub pronunciation: String,
    /// アクセント型。
    pub accent_type: i32,
    /// 単語の種類。
    pub word_type: UserDictWordType,
    /// 単語の優先度。
    pub priority: i32,
}

/// ユーザー辞書の単語の種類。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
