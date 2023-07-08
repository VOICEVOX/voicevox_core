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
        if std::path::Path::new(store_path).exists() {
            let store_file = File::open(store_path).map_err(|_| Error::InvalidDictFile)?;
            let words: HashMap<String, UserDictWord> =
                serde_json::from_reader(store_file).map_err(|_| Error::InvalidDictFile)?;
            serde_json::to_writer(&mut store_file, &words).map_err(|_| Error::InvalidDictFile)?;
            Ok(Self {
                store_path: store_path.to_string(),
                words,
            })
        } else {
            Ok(Self {
                store_path: store_path.to_string(),
                words: HashMap::new(),
            })
        }
    }

    pub fn add_word(&mut self, word: UserDictWord) -> Result<String> {
        let word_uuid = Uuid::new_v4().to_string();
        self.words.insert(word_uuid.clone(), word);
        self.save()?;
        Ok(word_uuid)
    }

    pub fn remove_word(&mut self, word_uuid: &str) -> Result<Option<UserDictWord>> {
        let word = self.words.remove(word_uuid);
        self.save()?;
        Ok(word)
    }

    fn save(&self) -> Result<()> {
        let mut file = File::create(&self.store_path).map_err(|_| Error::InvalidDictFile)?;
        serde_json::to_writer(&mut file, &self.words).map_err(|_| Error::InvalidDictFile)?;
        Ok(())
    }
}
