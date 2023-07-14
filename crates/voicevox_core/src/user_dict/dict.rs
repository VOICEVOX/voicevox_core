use derive_getters::Getters;
use std::{collections::HashMap, fs::File};
use uuid::Uuid;

use super::word::*;
use crate::{Error, Result};

/// ユーザー辞書。
#[derive(Clone, Debug, Default, Getters)]
pub struct UserDict {
    words: HashMap<Uuid, UserDictWord>,
}

impl UserDict {
    /// ユーザー辞書を作成する。
    pub fn new() -> Self {
        Default::default()
    }

    /// ユーザー辞書をファイルから読み込む。
    ///
    /// ファイルが存在しない、または内容が不正の場合はエラーを返す。
    pub fn load(&mut self, store_path: &str) -> Result<()> {
        let store_path = std::path::Path::new(store_path);
        if !store_path.exists() {
            return Err(Error::UserDictLoad("ファイルが存在しません".to_string()));
        }

        let store_file = File::open(store_path).map_err(|e| Error::UserDictLoad(e.to_string()))?;

        let words: HashMap<Uuid, UserDictWord> =
            serde_json::from_reader(store_file).map_err(|e| Error::UserDictLoad(e.to_string()))?;

        self.words.extend(words.into_iter());
        Ok(())
    }

    /// ユーザー辞書に単語を追加する。
    pub fn add_word(&mut self, word: UserDictWord) -> Result<Uuid> {
        let word_uuid = Uuid::new_v4();
        self.words.insert(word_uuid, word);
        Ok(word_uuid)
    }

    /// ユーザー辞書の単語を変更する。
    pub fn update_word(&mut self, word_uuid: Uuid, new_word: UserDictWord) -> Result<()> {
        if !self.words.contains_key(&word_uuid) {
            return Err(Error::WordNotFound(word_uuid));
        }
        self.words.insert(word_uuid, new_word);
        Ok(())
    }

    /// ユーザー辞書から単語を削除する。
    pub fn remove_word(&mut self, word_uuid: Uuid) -> Result<UserDictWord> {
        let Some(word) = self.words.remove(&word_uuid) else {
            return Err(Error::WordNotFound(word_uuid));
        };
        Ok(word)
    }

    /// 他のユーザー辞書をインポートする。
    pub fn import(&mut self, other: &Self) -> Result<()> {
        for (word_uuid, word) in &other.words {
            self.words.insert(word_uuid.to_owned(), word.clone());
        }
        Ok(())
    }

    /// ユーザー辞書を保存する。
    pub fn save(&self, store_path: &str) -> Result<()> {
        let mut file = File::create(store_path).map_err(|e| Error::UserDictSave(e.to_string()))?;
        serde_json::to_writer(&mut file, &self.words)
            .map_err(|e| Error::UserDictSave(e.to_string()))?;
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
