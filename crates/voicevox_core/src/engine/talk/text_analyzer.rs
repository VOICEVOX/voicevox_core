// TODO: 破壊的変更をするタイミングで、`analyze`として`enable_katakana_english`を指定可能にする(あとデフォルトを`true`にする)。

pub const DEFAULT_ENABLE_KATAKANA_ENGLISH: bool = false;

pub(crate) mod blocking {
    use crate::AccentPhrase;

    /// テキスト解析器。
    pub trait TextAnalyzer: Sync {
        /// テキストを解析する。
        fn analyze(&self, text: &str) -> anyhow::Result<Vec<AccentPhrase>>;

        /// `OpenTalk`専用。
        #[doc(hidden)]
        fn __analyze_with_options(
            &self,
            text: &str,
            #[allow(unused_variables)] enable_katakana_english: bool,
        ) -> anyhow::Result<Vec<AccentPhrase>> {
            self.analyze(text)
        }
    }
}

pub(crate) mod nonblocking {
    use std::future::Future;

    use crate::AccentPhrase;

    /// テキスト解析器。
    pub trait TextAnalyzer: Sync {
        /// テキストを解析する。
        fn analyze(
            &self,
            text: &str,
        ) -> impl Future<Output = anyhow::Result<Vec<AccentPhrase>>> + Send;

        /// `OpenTalk`専用。
        #[doc(hidden)]
        fn __analyze_with_options(
            &self,
            text: &str,
            #[allow(unused_variables)] enable_katakana_english: bool,
        ) -> impl Future<Output = anyhow::Result<Vec<AccentPhrase>>> + Send {
            self.analyze(text)
        }
    }
}
