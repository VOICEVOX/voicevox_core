use std::marker::PhantomData;

use educe::Educe;

pub const DEFAULT_ENABLE_KATAKANA_ENGLISH: bool = true;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Educe)]
#[educe(Default)]
#[non_exhaustive]
pub struct AnalyzeTextOptions<'a> {
    #[educe(Default(expression = "DEFAULT_ENABLE_KATAKANA_ENGLISH"))]
    pub enable_katakana_english: bool,

    pub _marker: PhantomData<&'a ()>,
}

impl AnalyzeTextOptions<'_> {
    pub fn enable_katakana_english(self, enable_katakana_english: bool) -> Self {
        Self {
            enable_katakana_english,
            ..self
        }
    }
}

pub(crate) mod blocking {
    use crate::AccentPhrase;

    use super::AnalyzeTextOptions;

    /// テキスト解析器。
    pub trait TextAnalyzer: Sync {
        /// テキストを解析する。
        fn analyze(
            &self,
            text: &str,
            options: AnalyzeTextOptions<'_>,
        ) -> anyhow::Result<Vec<AccentPhrase>>;
    }
}

pub(crate) mod nonblocking {
    use crate::AccentPhrase;

    use super::AnalyzeTextOptions;

    /// テキスト解析器。
    pub trait TextAnalyzer: Sync {
        /// テキストを解析する。
        fn analyze(
            &self,
            text: &str,
            options: AnalyzeTextOptions<'_>,
        ) -> impl Future<Output = anyhow::Result<Vec<AccentPhrase>>> + Send;
    }
}
