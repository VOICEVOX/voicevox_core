use derive_getters::Getters;
use std::io::Read;
use std::{collections::HashMap, fs::File};
use uuid::Uuid;

use super::word::*;
use crate::{Error, Result};

/// ユーザー辞書。
#[derive(Clone, Debug, Getters)]
pub struct UserDict {
    store_path: String,
    words: HashMap<String, UserDictWord>,
}

impl UserDict {
    /// ユーザー辞書をロードする。
    /// ファイルが存在しない場合は空の辞書を作成する。
    /// ファイルが存在する場合は、その内容を読み込む。ファイルの中身が無い場合は空の辞書を作成する。
    /// ファイルの内容が不正な場合はエラーを返す。
    pub fn new(store_path: &str) -> Result<Self> {
        if std::path::Path::new(store_path).exists() {
            let mut store_file =
                File::open(store_path).map_err(|e| Error::UserDictRead(e.to_string()))?;
            let mut content = String::new();
            store_file
                .read_to_string(&mut content)
                .map_err(|e| Error::UserDictRead(e.to_string()))?;

            let words: HashMap<String, UserDictWord> = if content.is_empty() {
                HashMap::new()
            } else {
                serde_json::from_str(&content[..]).map_err(|e| Error::UserDictRead(e.to_string()))?
            };
            Ok(Self {
                store_path: store_path.to_string(),
                words,
            })
        } else {
            let dict = Self {
                store_path: store_path.to_string(),
                words: HashMap::new(),
            };
            dict.save()?;
            Ok(dict)
        }
    }

    /// ユーザー辞書に単語を追加する。
    pub fn add_word(&mut self, word: UserDictWord) -> Result<String> {
        let word_uuid = Uuid::new_v4().to_string();
        self.words.insert(word_uuid.clone(), word);
        self.save()?;
        Ok(word_uuid)
    }

    /// ユーザー辞書の単語を変更する。
    pub fn update_word(&mut self, word_uuid: &str, new_word: UserDictWord) -> Result<()> {
        if !self.words.contains_key(word_uuid) {
            return Err(Error::WordNotFound(word_uuid.to_string()));
        }
        self.words.insert(word_uuid.to_string(), new_word);
        self.save()?;
        Ok(())
    }

    /// ユーザー辞書から単語を削除する。
    pub fn remove_word(&mut self, word_uuid: &str) -> Result<UserDictWord> {
        let Some(word) = self.words.remove(word_uuid) else {
            return Err(Error::WordNotFound(word_uuid.to_string()));
        };
        self.save()?;
        Ok(word)
    }

    /// 2つのユーザー辞書をマージする。
    pub fn merge(&mut self, other: &Self) -> Result<()> {
        for (word_uuid, word) in &other.words {
            self.words.insert(word_uuid.clone(), word.clone());
        }
        self.save()?;
        Ok(())
    }

    /// ユーザー辞書を保存する。
    fn save(&self) -> Result<()> {
        let mut file =
            File::create(&self.store_path).map_err(|e| Error::UserDictWrite(e.to_string()))?;
        serde_json::to_writer(&mut file, &self.words)
            .map_err(|e| Error::UserDictWrite(e.to_string()))?;
        Ok(())
    }

    /// MeCabで使用する形式に変換する。
    pub(crate) fn to_mecab_format(&self) -> String {
        let mut lines = Vec::new();
        for word in self.words.values() {
            lines.push(word.to_mecab_format());
        }
        lines.join("\n")
    }
}
