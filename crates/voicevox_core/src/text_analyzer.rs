pub(crate) mod blocking {
    use crate::AccentPhrase;

    pub trait TextAnalyzer: Sync {
        fn analyze(&self, text: &str) -> anyhow::Result<Vec<AccentPhrase>>;
    }
}

pub(crate) mod nonblocking {
    use std::future::Future;

    use crate::AccentPhrase;

    pub trait TextAnalyzer: Sync {
        fn analyze(
            &self,
            text: &str,
        ) -> impl Future<Output = anyhow::Result<Vec<AccentPhrase>>> + Send;
    }
}
