use derive_getters::Getters;
use fs_err::File;
use indexmap::IndexMap;
use itertools::join;
use uuid::Uuid;

use super::word::*;
use crate::{error::ErrorRepr, Result};

/// ユーザー辞書。
/// 単語はJSONとの相互変換のために挿入された順序を保つ。
#[derive(Clone, Debug, Default, Getters)]
pub struct UserDict {
    words: IndexMap<Uuid, UserDictWord>,
}

impl UserDict {
    /// ユーザー辞書を作成する。
    pub fn new() -> Self {
        Default::default()
    }

    /// ユーザー辞書をファイルから読み込む。
    ///
    /// # Errors
    ///
    /// ファイルが読めなかった、または内容が不正だった場合はエラーを返す。
    pub fn load(&mut self, store_path: &str) -> Result<()> {
        let store_path = std::path::Path::new(store_path);

        let store_file = File::open(store_path).map_err(|e| ErrorRepr::LoadUserDict(e.into()))?;

        let words: IndexMap<Uuid, UserDictWord> =
            serde_json::from_reader(store_file).map_err(|e| ErrorRepr::LoadUserDict(e.into()))?;

        self.words.extend(words);
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
            return Err(ErrorRepr::WordNotFound(word_uuid).into());
        }
        self.words.insert(word_uuid, new_word);
        Ok(())
    }

    /// ユーザー辞書から単語を削除する。
    pub fn remove_word(&mut self, word_uuid: Uuid) -> Result<UserDictWord> {
        let Some(word) = self.words.remove(&word_uuid) else {
            return Err(ErrorRepr::WordNotFound(word_uuid).into());
        };
        Ok(word)
    }

    /// 他のユーザー辞書をインポートする。
    pub fn import(&mut self, other: &Self) -> Result<()> {
        for (word_uuid, word) in &other.words {
            self.words.insert(*word_uuid, word.clone());
        }
        Ok(())
    }

    /// ユーザー辞書を保存する。
    pub fn save(&self, store_path: &str) -> Result<()> {
        let mut file = File::create(store_path).map_err(|e| ErrorRepr::SaveUserDict(e.into()))?;
        serde_json::to_writer(&mut file, &self.words)
            .map_err(|e| ErrorRepr::SaveUserDict(e.into()))?;
        Ok(())
    }

    /// MeCabで使用する形式に変換する。
    pub(crate) fn to_mecab_format(&self) -> String {
        join(self.words.values().map(UserDictWord::to_mecab_format), "\n")
    }
}
