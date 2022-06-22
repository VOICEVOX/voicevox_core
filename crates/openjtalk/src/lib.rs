use openjtalk_sys::*;

use std::ffi::CString;
use std::os::raw::c_void;
use std::path::{Path, PathBuf};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("Mecab load error: couldn't load mecab dictionary: {mecab_dictionary_dir}")]
    Load { mecab_dictionary_dir: PathBuf },
}

type Result<T> = std::result::Result<T, Error>;

pub struct OpenJTalk {
    ptr: *mut c_void,
}

impl Drop for OpenJTalk {
    fn drop(&mut self) {
        self.delete();
    }
}

impl OpenJTalk {
    pub fn create() -> Self {
        Self {
            ptr: unsafe { OpenJTalk_create() },
        }
    }

    pub fn load(&mut self, mecab_dictionary_dir: impl AsRef<Path>) -> Result<()> {
        let mecab_dictionary_dir = mecab_dictionary_dir.as_ref();
        let dn_mecab_cstr = CString::new(format!("{}", mecab_dictionary_dir.display())).unwrap();
        let res = unsafe { OpenJTalk_load(self.ptr, dn_mecab_cstr.as_ptr()) };
        if res == 0 {
            Ok(())
        } else {
            Err(Error::Load {
                mecab_dictionary_dir: mecab_dictionary_dir.into(),
            })
        }
    }

    pub fn extract_fullcontext(&self, text: impl AsRef<str>) -> Vec<String> {
        let text = CString::new(text.as_ref()).unwrap();
        let mut extract_size = 0;
        let mut result = Vec::new();
        unsafe {
            let labels_ptr = OpenJTalk_extract_fullcontext(
                self.ptr,
                text.as_ptr(),
                (&mut extract_size) as *mut usize,
            );
            for ptr in std::slice::from_raw_parts(labels_ptr, extract_size) {
                let c_str = CString::from_raw(*ptr);
                result.push(c_str.to_str().unwrap().to_string());
            }
        }
        result
    }

    pub fn clear(&mut self) {
        unsafe {
            OpenJTalk_clear(self.ptr);
        }
    }

    fn delete(&mut self) {
        unsafe {
            OpenJTalk_delete(self.ptr);
        }
    }
}
