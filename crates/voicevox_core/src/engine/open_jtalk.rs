use std::path::{Path, PathBuf};

use ::open_jtalk::*;

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

impl PartialEq for OpenJtalkError {
    fn eq(&self, other: &Self) -> bool {
        return match (self, other) {
            (
                Self::Load {
                    mecab_dict_dir: mecab_dict_dir1,
                },
                Self::Load {
                    mecab_dict_dir: mecab_dict_dir2,
                },
            ) => mecab_dict_dir1 == mecab_dict_dir2,
            (
                Self::ExtractFullContext {
                    text: text1,
                    source: source1,
                },
                Self::ExtractFullContext {
                    text: text2,
                    source: source2,
                },
            ) => (text1, by_display(source1)) == (text2, by_display(source2)),
            _ => false,
        };

        fn by_display(source: &Option<anyhow::Error>) -> impl PartialEq {
            source.as_ref().map(|e| e.to_string())
        }
    }
}

pub type Result<T> = std::result::Result<T, OpenJtalkError>;

pub struct OpenJtalk {
    mecab: ManagedResource<Mecab>,
    njd: ManagedResource<Njd>,
    jpcommon: ManagedResource<JpCommon>,
    dict_loaded: bool,
}

impl OpenJtalk {
    pub fn initialize() -> Self {
        Self {
            mecab: ManagedResource::initialize(),
            njd: ManagedResource::initialize(),
            jpcommon: ManagedResource::initialize(),
            dict_loaded: false,
        }
    }

    pub fn extract_fullcontext(&mut self, text: impl AsRef<str>) -> Result<Vec<String>> {
        let result = self.extract_fullcontext_non_reflesh(text);
        self.jpcommon.refresh();
        self.njd.refresh();
        self.mecab.refresh();
        result
    }

    fn extract_fullcontext_non_reflesh(&mut self, text: impl AsRef<str>) -> Result<Vec<String>> {
        let mecab_text =
            text2mecab(text.as_ref()).map_err(|e| OpenJtalkError::ExtractFullContext {
                text: text.as_ref().into(),
                source: Some(e.into()),
            })?;
        if self.mecab.analysis(mecab_text) {
            self.njd.mecab2njd(
                self.mecab
                    .get_feature()
                    .ok_or(OpenJtalkError::ExtractFullContext {
                        text: text.as_ref().into(),
                        source: None,
                    })?,
                self.mecab.get_size(),
            );
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

    pub fn load(&mut self, mecab_dict_dir: impl AsRef<Path>) -> Result<()> {
        let result = self.mecab.load(mecab_dict_dir.as_ref());
        if result {
            self.dict_loaded = true;
            Ok(())
        } else {
            self.dict_loaded = false;
            Err(OpenJtalkError::Load {
                mecab_dict_dir: mecab_dict_dir.as_ref().into(),
            })
        }
    }

    pub fn dict_loaded(&self) -> bool {
        self.dict_loaded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use test_util::OPEN_JTALK_DIC_DIR;

    use crate::*;

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
        let mut open_jtalk = OpenJtalk::initialize();
        open_jtalk.load(OPEN_JTALK_DIC_DIR).unwrap();
        let result = open_jtalk.extract_fullcontext(text);
        assert_eq!(expected, result);
    }

    #[rstest]
    #[case("こんにちは、ヒホです。", Ok(testdata_hello_hiho()))]
    fn extract_fullcontext_loop_works(
        #[case] text: &str,
        #[case] expected: super::Result<Vec<String>>,
    ) {
        let mut open_jtalk = OpenJtalk::initialize();
        open_jtalk.load(OPEN_JTALK_DIC_DIR).unwrap();
        for _ in 0..10 {
            let result = open_jtalk.extract_fullcontext(text);
            assert_eq!(expected, result);
        }
    }
}
