// JSから実装をimportするか、wasm対応する

pub(crate) mod blocking {
    use camino::Utf8Path;

    use crate::assert::assert_send_sync;

    use super::{
        super::{AccentPhrase, extract_full_context_label},
        FullcontextExtractor,
    };

    /// Link-test substitute; text analysis is intentionally unavailable.
    #[derive(Clone, Debug)]
    pub struct OpenJtalk;

    impl OpenJtalk {
        /// Constructs the substitute without loading a dictionary.
        pub fn new(_: impl AsRef<Utf8Path>) -> crate::result::Result<Self> {
            Ok(Self)
        }

        /// Accepts the API call without modifying a dictionary.
        pub fn use_user_dict(&self, _: &crate::blocking::UserDict) -> crate::result::Result<()> {
            Ok(())
        }
    }

    impl FullcontextExtractor for OpenJtalk {
        fn extract_fullcontext(&self, _text: &str) -> anyhow::Result<Vec<String>> {
            return Ok([
                    "xx^xx-sil+k=o/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:04+xx_xx/E:xx_xx!xx_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:3_3%0_xx_xx/H:xx_xx/I:xx-xx@xx+xx&xx-xx|xx+xx/J:2_8/K:1+2-8",
                    "xx^sil-k+o=r/A:-2+1+3/B:xx-xx_xx/C:04_xx+xx/D:24+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_2|1_8/G:5_1%0_xx_1/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "sil^k-o+r=e/A:-2+1+3/B:xx-xx_xx/C:04_xx+xx/D:24+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_2|1_8/G:5_1%0_xx_1/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "k^o-r+e=w/A:-1+2+2/B:xx-xx_xx/C:04_xx+xx/D:24+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_2|1_8/G:5_1%0_xx_1/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "o^r-e+w=a/A:-1+2+2/B:xx-xx_xx/C:04_xx+xx/D:24+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_2|1_8/G:5_1%0_xx_1/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "r^e-w+a=t/A:0+3+1/B:04-xx_xx/C:24_xx+xx/D:03+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_2|1_8/G:5_1%0_xx_1/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "e^w-a+t=e/A:0+3+1/B:04-xx_xx/C:24_xx+xx/D:03+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_2|1_8/G:5_1%0_xx_1/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "w^a-t+e=s/A:0+1+5/B:24-xx_xx/C:03_xx+xx/D:10+7_2/E:3_3!0_xx-1/F:5_1#0_xx@2_1|4_5/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "a^t-e+s=U/A:0+1+5/B:24-xx_xx/C:03_xx+xx/D:10+7_2/E:3_3!0_xx-1/F:5_1#0_xx@2_1|4_5/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "t^e-s+U=t/A:1+2+4/B:24-xx_xx/C:03_xx+xx/D:10+7_2/E:3_3!0_xx-1/F:5_1#0_xx@2_1|4_5/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "e^s-U+t=o/A:1+2+4/B:24-xx_xx/C:03_xx+xx/D:10+7_2/E:3_3!0_xx-1/F:5_1#0_xx@2_1|4_5/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "s^U-t+o=d/A:2+3+3/B:24-xx_xx/C:03_xx+xx/D:10+7_2/E:3_3!0_xx-1/F:5_1#0_xx@2_1|4_5/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "U^t-o+d=e/A:2+3+3/B:24-xx_xx/C:03_xx+xx/D:10+7_2/E:3_3!0_xx-1/F:5_1#0_xx@2_1|4_5/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "t^o-d+e=s/A:3+4+2/B:03-xx_xx/C:10_7+2/D:xx+xx_xx/E:3_3!0_xx-1/F:5_1#0_xx@2_1|4_5/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "o^d-e+s=U/A:3+4+2/B:03-xx_xx/C:10_7+2/D:xx+xx_xx/E:3_3!0_xx-1/F:5_1#0_xx@2_1|4_5/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "d^e-s+U=sil/A:4+5+1/B:03-xx_xx/C:10_7+2/D:xx+xx_xx/E:3_3!0_xx-1/F:5_1#0_xx@2_1|4_5/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "e^s-U+sil=xx/A:4+5+1/B:03-xx_xx/C:10_7+2/D:xx+xx_xx/E:3_3!0_xx-1/F:5_1#0_xx@2_1|4_5/G:xx_xx%xx_xx_xx/H:xx_xx/I:2-8@1+1&1-2|1+8/J:xx_xx/K:1+2-8",
                    "s^U-sil+xx=xx/A:xx+xx+xx/B:10-7_2/C:xx_xx+xx/D:xx+xx_xx/E:5_1!0_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:xx_xx%xx_xx_xx/H:2_8/I:xx-xx@xx+xx&xx-xx|xx+xx/J:xx_xx/K:1+2-8",
                ]
                .into_iter()
                .map(str::to_owned)
                .collect());
        }
    }

    impl crate::blocking::TextAnalyzer for OpenJtalk {
        fn analyze(&self, text: &str) -> anyhow::Result<Vec<AccentPhrase>> {
            if text.is_empty() {
                return Ok(Vec::new());
            }
            Ok(extract_full_context_label(self, text)?)
        }
    }

    assert_send_sync!(OpenJtalk);
}

pub(crate) mod nonblocking {
    use camino::Utf8Path;

    use crate::assert::assert_send_sync;

    /// Asynchronous wrapper for the link-test substitute.
    #[derive(Clone, Debug)]
    pub struct OpenJtalk(pub(in super::super) super::blocking::OpenJtalk);

    impl OpenJtalk {
        /// Constructs the substitute without loading a dictionary.
        pub async fn new(open_jtalk_dict_dir: impl AsRef<Utf8Path>) -> crate::result::Result<Self> {
            super::blocking::OpenJtalk::new(open_jtalk_dict_dir).map(Self)
        }

        /// Accepts the API call without modifying a dictionary.
        pub async fn use_user_dict(
            &self,
            _: &crate::nonblocking::UserDict,
        ) -> crate::result::Result<()> {
            Ok(())
        }
    }

    impl crate::nonblocking::TextAnalyzer for OpenJtalk {
        async fn analyze(&self, text: &str) -> anyhow::Result<Vec<crate::AccentPhrase>> {
            crate::blocking::TextAnalyzer::analyze(&self.0, text)
        }
    }

    assert_send_sync!(OpenJtalk);
}
