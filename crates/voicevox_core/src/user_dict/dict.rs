use indexmap::IndexMap;
use itertools::join;
use uuid::Uuid;

use super::word::*;
use crate::{error::ErrorRepr, Result};

/// ユーザー辞書。
/// 単語はJSONとの相互変換のために挿入された順序を保つ。
#[derive(Debug, Default)]
pub struct UserDict {
    words: std::sync::Mutex<IndexMap<Uuid, UserDictWord>>,
}

impl UserDict {
    /// ユーザー辞書を作成する。
    pub fn new() -> Self {
        Default::default()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&*self.words.lock().unwrap()).expect("should not fail")
    }

    pub fn with_words<R>(&self, f: impl FnOnce(&IndexMap<Uuid, UserDictWord>) -> R) -> R {
        f(&self.words.lock().unwrap())
    }

    /// ユーザー辞書をファイルから読み込む。
    ///
    /// # Errors
    ///
    /// ファイルが読めなかった、または内容が不正だった場合はエラーを返す。
    pub async fn load(&self, store_path: &str) -> Result<()> {
        let words = async {
            let words = &fs_err::tokio::read(store_path).await?;
            let words = serde_json::from_slice::<IndexMap<_, _>>(words)?;
            Ok(words)
        }
        .await
        .map_err(ErrorRepr::LoadUserDict)?;

        self.words.lock().unwrap().extend(words);
        Ok(())
    }

    /// ユーザー辞書に単語を追加する。
    pub fn add_word(&self, word: UserDictWord) -> Result<Uuid> {
        let word_uuid = Uuid::new_v4();
        self.words.lock().unwrap().insert(word_uuid, word);
        Ok(word_uuid)
    }

    /// ユーザー辞書の単語を変更する。
    pub fn update_word(&self, word_uuid: Uuid, new_word: UserDictWord) -> Result<()> {
        let mut words = self.words.lock().unwrap();
        if !words.contains_key(&word_uuid) {
            return Err(ErrorRepr::WordNotFound(word_uuid).into());
        }
        words.insert(word_uuid, new_word);
        Ok(())
    }

    /// ユーザー辞書から単語を削除する。
    pub fn remove_word(&self, word_uuid: Uuid) -> Result<UserDictWord> {
        let Some(word) = self.words.lock().unwrap().remove(&word_uuid) else {
            return Err(ErrorRepr::WordNotFound(word_uuid).into());
        };
        Ok(word)
    }

    /// 他のユーザー辞書をインポートする。
    pub fn import(&self, other: &Self) -> Result<()> {
        for (word_uuid, word) in &*other.words.lock().unwrap() {
            self.words.lock().unwrap().insert(*word_uuid, word.clone());
        }
        Ok(())
    }

    /// ユーザー辞書を保存する。
    pub async fn save(&self, store_path: &str) -> Result<()> {
        fs_err::tokio::write(
            store_path,
            serde_json::to_vec(&self.words).expect("should not fail"),
        )
        .await
        .map_err(|e| ErrorRepr::SaveUserDict(e.into()).into())
    }

    /// MeCabで使用する形式に変換する。
    pub(crate) fn to_mecab_format(&self) -> String {
        join(
            self.words
                .lock()
                .unwrap()
                .values()
                .map(UserDictWord::to_mecab_format),
            "\n",
        )
    }
}
