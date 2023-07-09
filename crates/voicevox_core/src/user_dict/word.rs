use crate::user_dict::part_of_speech_data::word_type_to_part_of_speech_detail;
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
    pub accent_type: usize,
    /// 単語の種類。
    pub word_type: UserDictWordType,
    /// 単語の優先度。
    pub priority: u32,
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

impl UserDictWord {
    pub fn to_mecab_format(&self) -> String {
        let pos = word_type_to_part_of_speech_detail(&self.word_type);
        vec![
            self.surface.clone(),
            self.surface.clone(),
            self.priority.to_string(),
            pos.part_of_speech,
            pos.part_of_speech_detail_1,
            pos.part_of_speech_detail_2,
            pos.part_of_speech_detail_3,
            "*".to_string(), // pos.inflectional_type,
            "*".to_string(), // pos.inflectional_form,
            self.pronunciation.clone(),
            self.pronunciation.clone(),
            self.accent_type.to_string(),
            self.pronunciation.chars().count().to_string(),
            "0".to_string(),
        ]
        .join(",")
    }
}
