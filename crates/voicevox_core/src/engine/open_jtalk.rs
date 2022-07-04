use std::path::{Path, PathBuf};

use ::open_jtalk::*;

/*
 * TODO: OpenJtalk機能を使用するようになったら、allow(dead_code)を消す
 */

#[allow(dead_code)]
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum OpenJtalkError {
    #[error("open_jtalk load error")]
    Load { mecab_dict_dir: PathBuf },
    #[error("open_jtalk extract_fullcontext error")]
    ExtractFullContext {
        text: String,
        #[source]
        source: Option<crate::error::SourceError>,
    },
}

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, OpenJtalkError>;

#[allow(dead_code)]
pub struct OpenJtalk {
    mecab: ManagedResource<Mecab>,
    njd: ManagedResource<Njd>,
    jpcommon: ManagedResource<JpCommon>,
    dict_loaded: bool,
}

impl OpenJtalk {
    #[allow(dead_code)]
    pub fn initialize() -> Self {
        Self {
            mecab: ManagedResource::initialize(),
            njd: ManagedResource::initialize(),
            jpcommon: ManagedResource::initialize(),
            dict_loaded: false,
        }
    }

    #[allow(dead_code)]
    pub fn extract_fullcontext(&mut self, text: impl AsRef<str>) -> Result<Vec<String>> {
        let mecab_text =
            text2mecab(text.as_ref()).map_err(|e| OpenJtalkError::ExtractFullContext {
                text: text.as_ref().into(),
                source: Some(e.into()),
            })?;
        if self.mecab.analysis(mecab_text) {
            self.njd.set_pronunciation();
            self.njd.set_digit();
            self.njd.set_accent_phrase();
            self.njd.set_accent_type();
            self.njd.set_unvoiced_vowel();
            self.njd.set_long_vowel();
            self.jpcommon.njd2jpcommon(&self.njd);
            self.jpcommon.make_label();
            self.jpcommon
                .get_label_feature_to_iter()
                .ok_or_else(|| OpenJtalkError::ExtractFullContext {
                    text: text.as_ref().into(),
                    source: None,
                })
                .map(|iter| iter.map(|s| s.to_string()).collect())
        } else {
            Err(OpenJtalkError::ExtractFullContext {
                text: text.as_ref().into(),
                source: None,
            })
        }
    }

    #[allow(dead_code)]
    pub fn load(&mut self, mecab_dict_dir: impl AsRef<Path>) -> Result<()> {
        let result = self.mecab.load(mecab_dict_dir.as_ref());
        if result {
            self.dict_loaded = true;
            Ok(())
        } else {
            Err(OpenJtalkError::Load {
                mecab_dict_dir: mecab_dict_dir.as_ref().into(),
            })
        }
    }
}
