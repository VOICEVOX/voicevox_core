use std::{marker::PhantomData, path::Path};

use anyhow::Context as _;
use easy_ext::ext;
use educe::Educe;
use indexmap::IndexMap;
use itertools::Itertools as _;
use uuid::Uuid;

use crate::{asyncs::Async, error::ErrorRepr};

use super::UserDictWord;

#[derive(Educe)]
#[educe(Default(bound = "A:"))]
#[educe(Debug(bound = "A:"))]
struct Inner<A> {
    words: std::sync::Mutex<IndexMap<Uuid, UserDictWord>>,
    _marker: PhantomData<A>,
}

impl<A: Async> Inner<A> {
    fn to_json(&self) -> String {
        self.with_words(|words| serde_json::to_string(words).expect("should not fail"))
    }

    fn with_words<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut IndexMap<Uuid, UserDictWord>) -> R,
    {
        f(&mut self.words.lock().unwrap_or_else(|e| panic!("{e}")))
    }

    async fn load(&self, store_path: &str) -> crate::Result<()> {
        let words = async {
            let words = &A::fs_err_read(store_path).await?;
            let words = serde_json::from_slice::<IndexMap<_, _>>(words)?;
            Ok(words)
        }
        .await
        .map_err(ErrorRepr::LoadUserDict)?;

        self.with_words(|words_| words_.extend(words));
        Ok(())
    }

    fn add_word(&self, word: UserDictWord) -> crate::Result<Uuid> {
        let word_uuid = Uuid::new_v4();
        self.with_words(|word_| word_.insert(word_uuid, word));
        Ok(word_uuid)
    }

    fn update_word(&self, word_uuid: Uuid, new_word: UserDictWord) -> crate::Result<()> {
        self.with_words(|words| {
            if !words.contains_key(&word_uuid) {
                return Err(ErrorRepr::WordNotFound(word_uuid).into());
            }
            words.insert(word_uuid, new_word);
            Ok(())
        })
    }

    fn remove_word(&self, word_uuid: Uuid) -> crate::Result<UserDictWord> {
        let Some(word) = self.with_words(|words| words.remove(&word_uuid)) else {
            return Err(ErrorRepr::WordNotFound(word_uuid).into());
        };
        Ok(word)
    }

    fn import(&self, other: &Self) -> crate::Result<()> {
        self.with_words(|self_words| {
            other.with_words(|other_words| {
                for (word_uuid, word) in other_words {
                    self_words.insert(*word_uuid, word.clone());
                }
                Ok(())
            })
        })
    }

    async fn save(&self, store_path: &str) -> crate::Result<()> {
        A::fs_err_write(
            store_path,
            serde_json::to_vec(&self.words).expect("should not fail"),
        )
        .await
        .map_err(ErrorRepr::SaveUserDict)
        .map_err(Into::into)
    }

    fn to_mecab_format(&self) -> String {
        self.with_words(|words| words.values().map(UserDictWord::to_mecab_format).join("\n"))
    }
}

#[ext]
impl<A: Async> A {
    async fn fs_err_read(path: impl AsRef<Path>) -> anyhow::Result<Vec<u8>> {
        Self::read(&path)
            .await
            .with_context(|| format!("failed to read from file `{}`", path.as_ref().display()))
    }

    async fn fs_err_write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> anyhow::Result<()> {
        Self::write(&path, content)
            .await
            .with_context(|| format!("failed to write to file `{}`", path.as_ref().display()))
    }
}

pub(crate) mod blocking {
    use indexmap::IndexMap;
    use uuid::Uuid;

    use crate::{asyncs::SingleTasked, future::FutureExt as _, Result};

    use super::{super::word::UserDictWord, Inner};

    /// ユーザー辞書。
    ///
    /// 単語はJSONとの相互変換のために挿入された順序を保つ。
    #[derive(Debug, Default)]
    pub struct UserDict(Inner<SingleTasked>);

    // TODO: 引数の`path`は全部`AsRef<Path>`にする
    impl self::UserDict {
        /// ユーザー辞書を作成する。
        pub fn new() -> Self {
            Default::default()
        }

        pub fn to_json(&self) -> String {
            self.0.to_json()
        }

        // TODO: `&mut IndexMap<_>`を取れるようにする
        pub fn with_words<R>(&self, f: impl FnOnce(&IndexMap<Uuid, UserDictWord>) -> R) -> R {
            self.0.with_words(|words| f(words))
        }

        /// ユーザー辞書をファイルから読み込む。
        ///
        /// # Errors
        ///
        /// ファイルが読めなかった、または内容が不正だった場合はエラーを返す。
        pub fn load(&self, store_path: &str) -> Result<()> {
            self.0.load(store_path).block_on()
        }

        /// ユーザー辞書に単語を追加する。
        pub fn add_word(&self, word: UserDictWord) -> Result<Uuid> {
            self.0.add_word(word)
        }

        /// ユーザー辞書の単語を変更する。
        pub fn update_word(&self, word_uuid: Uuid, new_word: UserDictWord) -> Result<()> {
            self.0.update_word(word_uuid, new_word)
        }

        /// ユーザー辞書から単語を削除する。
        pub fn remove_word(&self, word_uuid: Uuid) -> Result<UserDictWord> {
            self.0.remove_word(word_uuid)
        }

        /// 他のユーザー辞書をインポートする。
        pub fn import(&self, other: &Self) -> Result<()> {
            self.0.import(&other.0)
        }

        /// ユーザー辞書を保存する。
        pub fn save(&self, store_path: &str) -> Result<()> {
            self.0.save(store_path).block_on()
        }

        /// MeCabで使用する形式に変換する。
        pub(crate) fn to_mecab_format(&self) -> String {
            self.0.to_mecab_format()
        }
    }
}

pub(crate) mod nonblocking {
    use indexmap::IndexMap;
    use uuid::Uuid;

    use crate::{asyncs::BlockingThreadPool, Result};

    use super::{super::word::UserDictWord, Inner};

    /// ユーザー辞書。
    ///
    /// 単語はJSONとの相互変換のために挿入された順序を保つ。
    ///
    /// # Performance
    ///
    /// [blocking]クレートにより動いている。詳しくは[`nonblocking`モジュールのドキュメント]を参照。
    ///
    /// [blocking]: https://docs.rs/crate/blocking
    /// [`nonblocking`モジュールのドキュメント]: crate::nonblocking
    #[derive(Debug, Default)]
    pub struct UserDict(Inner<BlockingThreadPool>);

    // TODO: 引数の`path`は全部`AsRef<Path>`にする
    impl self::UserDict {
        /// ユーザー辞書を作成する。
        pub fn new() -> Self {
            Default::default()
        }

        pub fn to_json(&self) -> String {
            self.0.to_json()
        }

        // TODO: `&mut IndexMap<_>`を取れるようにする
        pub fn with_words<R>(&self, f: impl FnOnce(&IndexMap<Uuid, UserDictWord>) -> R) -> R {
            self.0.with_words(|words| f(words))
        }

        /// ユーザー辞書をファイルから読み込む。
        ///
        /// # Errors
        ///
        /// ファイルが読めなかった、または内容が不正だった場合はエラーを返す。
        pub async fn load(&self, store_path: &str) -> Result<()> {
            self.0.load(store_path).await
        }

        /// ユーザー辞書に単語を追加する。
        pub fn add_word(&self, word: UserDictWord) -> Result<Uuid> {
            self.0.add_word(word)
        }

        /// ユーザー辞書の単語を変更する。
        pub fn update_word(&self, word_uuid: Uuid, new_word: UserDictWord) -> Result<()> {
            self.0.update_word(word_uuid, new_word)
        }

        /// ユーザー辞書から単語を削除する。
        pub fn remove_word(&self, word_uuid: Uuid) -> Result<UserDictWord> {
            self.0.remove_word(word_uuid)
        }

        /// 他のユーザー辞書をインポートする。
        pub fn import(&self, other: &Self) -> Result<()> {
            self.0.import(&other.0)
        }

        /// ユーザー辞書を保存する。
        pub async fn save(&self, store_path: &str) -> Result<()> {
            self.0.save(store_path).await
        }

        /// MeCabで使用する形式に変換する。
        pub(crate) fn to_mecab_format(&self) -> String {
            self.0.to_mecab_format()
        }
    }
}
