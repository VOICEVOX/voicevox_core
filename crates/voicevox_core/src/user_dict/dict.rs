use derive_getters::Getters;
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
    /// ファイルが存在する場合は、その内容を読み込む。
    /// ファイルの内容が不正な場合はエラーを返す。
    pub fn new(store_path: &str) -> Result<Self> {
        if std::path::Path::new(store_path).exists() {
            let store_file = File::open(store_path).map_err(|_| Error::UserDictRead)?;
            let words: HashMap<String, UserDictWord> =
                serde_json::from_reader(store_file).map_err(|_| Error::UserDictRead)?;
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
    pub fn alter_word(&mut self, word_uuid: &str, new_word: UserDictWord) -> Result<()> {
        if !self.words.contains_key(word_uuid) {
            return Err(Error::WordNotFound);
        }
        self.words.insert(word_uuid.to_string(), new_word);
        self.save()?;
        Ok(())
    }

    /// ユーザー辞書から単語を削除する。
    pub fn remove_word(&mut self, word_uuid: &str) -> Result<UserDictWord> {
        let Some(word) = self.words.remove(word_uuid) else {
            return Err(Error::WordNotFound);
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
        let mut file = File::create(&self.store_path).map_err(|_| Error::UserDictWrite)?;
        serde_json::to_writer(&mut file, &self.words).map_err(|_| Error::UserDictWrite)?;
        Ok(())
    }
}
