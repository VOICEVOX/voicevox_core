use std::io::Write;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use tempfile::NamedTempFile;

use ::open_jtalk::*;

use crate::{Error, UserDict};

#[derive(thiserror::Error, Debug)]
pub enum OpenJtalkError {
    #[error("open_jtalk load error")]
    Load { mecab_dict_dir: PathBuf },
    #[error("open_jtalk extract_fullcontext error")]
    ExtractFullContext {
        text: String,
        #[source]
        source: Option<anyhow::Error>,
    },
}

pub type Result<T> = std::result::Result<T, OpenJtalkError>;

pub struct OpenJtalk {
    resources: Mutex<Resources>,
    dict_dir: Option<PathBuf>,
}

struct Resources {
    mecab: ManagedResource<Mecab>,
    njd: ManagedResource<Njd>,
    jpcommon: ManagedResource<JpCommon>,
}

#[allow(unsafe_code)]
unsafe impl Send for Resources {}

impl OpenJtalk {
    pub fn new_without_dic() -> Self {
        Self {
            resources: Mutex::new(Resources {
                mecab: ManagedResource::initialize(),
                njd: ManagedResource::initialize(),
                jpcommon: ManagedResource::initialize(),
            }),
            dict_dir: None,
        }
    }
    pub fn new_with_initialize(
        open_jtalk_dict_dir: impl AsRef<Path>,
    ) -> crate::result::Result<Self> {
        let mut s = Self::new_without_dic();
        s.load(open_jtalk_dict_dir)
            .map_err(|_| Error::NotLoadedOpenjtalkDict)?;
        Ok(s)
    }

    pub fn load_user_dict(&self, user_dict: &UserDict) -> crate::result::Result<()> {
        let dict_dir = self
            .dict_dir
            .as_ref()
            .and_then(|dict_dir| dict_dir.to_str())
            .ok_or(Error::NotLoadedOpenjtalkDict)?;
        let mut temp_csv = NamedTempFile::new().map_err(|e| Error::UserDictLoad(e.to_string()))?;
        temp_csv
            .write_all(user_dict.to_mecab_format().as_bytes())
            .map_err(|e| Error::UserDictLoad(e.to_string()))?;
        let temp_csv_path = temp_csv.into_temp_path();
        let temp_dict = NamedTempFile::new().map_err(|e| Error::UserDictLoad(e.to_string()))?;
        let temp_dict_path = temp_dict.into_temp_path();

        // TODO: エラー（SEGV）を良い感じに処理する
        mecab_dict_index(&[
            "mecab-dict-index",
            "-d",
            dict_dir,
            "-u",
            temp_dict_path.to_str().unwrap(),
            "-f",
            "utf-8",
            "-t",
            "utf-8",
            temp_csv_path.to_str().unwrap(),
            "-q",
        ]);

        let Resources { mecab, .. } = &mut *self.resources.lock().unwrap();

        let result = mecab.load_with_userdic(dict_dir, temp_dict_path.to_str().unwrap());

        if !result {
            return Err(Error::UserDictLoad(
                "辞書のコンパイルに失敗しました".to_string(),
            ));
        }

        Ok(())
    }

    pub fn extract_fullcontext(&self, text: impl AsRef<str>) -> Result<Vec<String>> {
        let Resources {
            mecab,
            njd,
            jpcommon,
        } = &mut *self.resources.lock().unwrap();

        jpcommon.refresh();
        njd.refresh();
        mecab.refresh();

        let mecab_text =
            text2mecab(text.as_ref()).map_err(|e| OpenJtalkError::ExtractFullContext {
                text: text.as_ref().into(),
                source: Some(e.into()),
            })?;
        if mecab.analysis(mecab_text) {
            njd.mecab2njd(
                mecab
                    .get_feature()
                    .ok_or(OpenJtalkError::ExtractFullContext {
                        text: text.as_ref().into(),
                        source: None,
                    })?,
                mecab.get_size(),
            );
            njd.set_pronunciation();
            njd.set_digit();
            njd.set_accent_phrase();
            njd.set_accent_type();
            njd.set_unvoiced_vowel();
            njd.set_long_vowel();
            jpcommon.njd2jpcommon(njd);
            jpcommon.make_label();
            jpcommon
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

    fn load(&mut self, open_jtalk_dict_dir: impl AsRef<Path>) -> Result<()> {
        let result = self
            .resources
            .lock()
            .unwrap()
            .mecab
            .load(open_jtalk_dict_dir.as_ref());
        if result {
            self.dict_dir = Some(open_jtalk_dict_dir.as_ref().into());
            Ok(())
        } else {
            self.dict_dir = None;
            Err(OpenJtalkError::Load {
                mecab_dict_dir: open_jtalk_dict_dir.as_ref().into(),
            })
        }
    }

    pub fn dict_loaded(&self) -> bool {
        self.dict_dir.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::test_util::OPEN_JTALK_DIC_DIR;

    use crate::{macros::tests::assert_debug_fmt_eq, *};

    fn testdata_hello_hiho() -> Vec<String> {
        // こんにちは、ヒホです。の期待値
        vec![
            // sil (無音)
            String::from(
                "xx^xx-sil+k=o/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx",
            ) + "/F:xx_xx#xx_xx@xx_xx|xx_xx/G:5_5%0_xx_xx/H:xx_xx/I:xx-xx"
                + "@xx+xx&xx-xx|xx+xx/J:1_5/K:2+2-9",
            // k
            String::from("xx^sil-k+o=N/A:-4+1+5/B:xx-xx_xx/C:09_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx")
                + "/F:5_5#0_xx@1_1|1_5/G:4_1%0_xx_0/H:xx_xx/I:1-5"
                + "@1+2&1-2|1+9/J:1_4/K:2+2-9",
            // o
            String::from("sil^k-o+N=n/A:-4+1+5/B:xx-xx_xx/C:09_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx")
                + "/F:5_5#0_xx@1_1|1_5/G:4_1%0_xx_0/H:xx_xx/I:1-5"
                + "@1+2&1-2|1+9/J:1_4/K:2+2-9",
            // N (ん)
            String::from("k^o-N+n=i/A:-3+2+4/B:xx-xx_xx/C:09_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx")
                + "/F:5_5#0_xx@1_1|1_5/G:4_1%0_xx_0/H:xx_xx/I:1-5"
                + "@1+2&1-2|1+9/J:1_4/K:2+2-9",
            // n
            String::from("o^N-n+i=ch/A:-2+3+3/B:xx-xx_xx/C:09_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx")
                + "/F:5_5#0_xx@1_1|1_5/G:4_1%0_xx_0/H:xx_xx/I:1-5"
                + "@1+2&1-2|1+9/J:1_4/K:2+2-9",
            // i
            String::from("N^n-i+ch=i/A:-2+3+3/B:xx-xx_xx/C:09_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx")
                + "/F:5_5#0_xx@1_1|1_5/G:4_1%0_xx_0/H:xx_xx/I:1-5"
                + "@1+2&1-2|1+9/J:1_4/K:2+2-9",
            // ch
            String::from("n^i-ch+i=w/A:-1+4+2/B:xx-xx_xx/C:09_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx")
                + "/F:5_5#0_xx@1_1|1_5/G:4_1%0_xx_0/H:xx_xx/I:1-5"
                + "@1+2&1-2|1+9/J:1_4/K:2+2-9",
            // i
            String::from("i^ch-i+w=a/A:-1+4+2/B:xx-xx_xx/C:09_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx")
                + "/F:5_5#0_xx@1_1|1_5/G:4_1%0_xx_0/H:xx_xx/I:1-5"
                + "@1+2&1-2|1+9/J:1_4/K:2+2-9",
            // w
            String::from("ch^i-w+a=pau/A:0+5+1/B:xx-xx_xx/C:09_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx")
                + "/F:5_5#0_xx@1_1|1_5/G:4_1%0_xx_0/H:xx_xx/I:1-5"
                + "@1+2&1-2|1+9/J:1_4/K:2+2-9",
            // a
            String::from("i^w-a+pau=h/A:0+5+1/B:xx-xx_xx/C:09_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx")
                + "/F:5_5#0_xx@1_1|1_5/G:4_1%0_xx_0/H:xx_xx/I:1-5"
                + "@1+2&1-2|1+9/J:1_4/K:2+2-9",
            // pau (読点)
            String::from("w^a-pau+h=i/A:xx+xx+xx/B:09-xx_xx/C:xx_xx+xx/D:09+xx_xx/E:5_5!0_xx-xx")
                + "/F:xx_xx#xx_xx@xx_xx|xx_xx/G:4_1%0_xx_xx/H:1_5/I:xx-xx"
                + "@xx+xx&xx-xx|xx+xx/J:1_4/K:2+2-9",
            // h
            String::from("a^pau-h+i=h/A:0+1+4/B:09-xx_xx/C:09_xx+xx/D:22+xx_xx/E:5_5!0_xx-0")
                + "/F:4_1#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_5/I:1-4"
                + "@2+1&2-1|6+4/J:xx_xx/K:2+2-9",
            // i
            String::from("pau^h-i+h=o/A:0+1+4/B:09-xx_xx/C:09_xx+xx/D:22+xx_xx/E:5_5!0_xx-0")
                + "/F:4_1#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_5/I:1-4"
                + "@2+1&2-1|6+4/J:xx_xx/K:2+2-9",
            // h
            String::from("h^i-h+o=d/A:1+2+3/B:09-xx_xx/C:22_xx+xx/D:10+7_2/E:5_5!0_xx-0")
                + "/F:4_1#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_5/I:1-4"
                + "@2+1&2-1|6+4/J:xx_xx/K:2+2-9",
            // o
            String::from("i^h-o+d=e/A:1+2+3/B:09-xx_xx/C:22_xx+xx/D:10+7_2/E:5_5!0_xx-0")
                + "/F:4_1#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_5/I:1-4"
                + "@2+1&2-1|6+4/J:xx_xx/K:2+2-9",
            // d
            String::from("h^o-d+e=s/A:2+3+2/B:22-xx_xx/C:10_7+2/D:xx+xx_xx/E:5_5!0_xx-0")
                + "/F:4_1#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_5/I:1-4"
                + "@2+1&2-1|6+4/J:xx_xx/K:2+2-9",
            // e
            String::from("o^d-e+s=U/A:2+3+2/B:22-xx_xx/C:10_7+2/D:xx+xx_xx/E:5_5!0_xx-0")
                + "/F:4_1#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_5/I:1-4"
                + "@2+1&2-1|6+4/J:xx_xx/K:2+2-9",
            // s
            String::from("d^e-s+U=sil/A:3+4+1/B:22-xx_xx/C:10_7+2/D:xx+xx_xx/E:5_5!0_xx-0")
                + "/F:4_1#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_5/I:1-4"
                + "@2+1&2-1|6+4/J:xx_xx/K:2+2-9",
            // U (無声母音)
            String::from("e^s-U+sil=xx/A:3+4+1/B:22-xx_xx/C:10_7+2/D:xx+xx_xx/E:5_5!0_xx-0")
                + "/F:4_1#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_5/I:1-4"
                + "@2+1&2-1|6+4/J:xx_xx/K:2+2-9",
            // sil (無音)
            String::from("s^U-sil+xx=xx/A:xx+xx+xx/B:10-7_2/C:xx_xx+xx/D:xx+xx_xx/E:4_1!0_xx-xx")
                + "/F:xx_xx#xx_xx@xx_xx|xx_xx/G:xx_xx%xx_xx_xx/H:1_4/I:xx-xx"
                + "@xx+xx&xx-xx|xx+xx/J:xx_xx/K:2+2-9",
        ]
    }

    #[rstest]
    #[case("",Err(OpenJtalkError::ExtractFullContext{text:"".into(),source:None}))]
    #[case("こんにちは、ヒホです。", Ok(testdata_hello_hiho()))]
    fn extract_fullcontext_works(#[case] text: &str, #[case] expected: super::Result<Vec<String>>) {
        let open_jtalk = OpenJtalk::new_with_initialize(OPEN_JTALK_DIC_DIR).unwrap();
        let result = open_jtalk.extract_fullcontext(text);
        assert_debug_fmt_eq!(expected, result);
    }

    #[rstest]
    #[case("こんにちは、ヒホです。", Ok(testdata_hello_hiho()))]
    fn extract_fullcontext_loop_works(
        #[case] text: &str,
        #[case] expected: super::Result<Vec<String>>,
    ) {
        let open_jtalk = OpenJtalk::new_with_initialize(OPEN_JTALK_DIC_DIR).unwrap();
        for _ in 0..10 {
            let result = open_jtalk.extract_fullcontext(text);
            assert_debug_fmt_eq!(expected, result);
        }
    }
}
