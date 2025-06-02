pub(crate) mod blocking {
    use crate::AccentPhrase;

    /// テキスト解析器。
    pub trait TextAnalyzer: Sync {
        /// テキストを解析する。
        fn analyze(&self, text: &str) -> anyhow::Result<Vec<AccentPhrase>>;
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
    }
}
