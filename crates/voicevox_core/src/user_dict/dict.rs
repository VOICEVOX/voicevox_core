use derive_getters::Getters;
use std::{collections::HashMap, fs::File};
use uuid::Uuid;

use super::word::*;
use crate::{Error, Result};

#[derive(Clone, Debug, Getters)]
pub struct UserDict {
    store_path: String,
    words: HashMap<String, UserDictWord>,
}

impl UserDict {
    pub fn new(store_path: &str) -> Result<Self> {
        let store_file = File::open(store_path).map_err(|_| Error::InvalidDictFile)?;
        let words: HashMap<String, UserDictWord> =
            serde_json::from_reader(store_file).map_err(|_| Error::InvalidDictFile)?;

        Ok(Self {
            store_path: store_path.to_string(),
            words,
        })
    }

    pub fn add_word(&mut self, word: UserDictWord) -> String {
        let word_uuid = Uuid::new_v4().to_string();
        self.words.insert(word_uuid.clone(), word);
        word_uuid
    }

    pub fn remove_word(&mut self, word_uuid: &str) -> Option<UserDictWord> {
        self.words.remove(word_uuid)
    }
}
