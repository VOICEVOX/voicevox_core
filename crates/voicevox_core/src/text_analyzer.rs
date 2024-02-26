use crate::{
    engine::{extract_full_context_label, parse_kana},
    AccentPhraseModel, FullcontextExtractor, Result,
};

pub trait TextAnalyzer {
    fn analyze(&self, text: &str) -> Result<Vec<AccentPhraseModel>>;
}

/// AquesTalk風記法からAccentPhraseの配列を生成するTextAnalyzer
#[derive(Clone)]
pub struct KanaAnalyzer;

impl TextAnalyzer for KanaAnalyzer {
    fn analyze(&self, text: &str) -> Result<Vec<AccentPhraseModel>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }
        Ok(parse_kana(text)?)
    }
}

/// OpenJtalkからAccentPhraseの配列を生成するTextAnalyzer
#[derive(Clone)]
pub struct OpenJTalkAnalyzer<O>(O);

impl<O> OpenJTalkAnalyzer<O> {
    pub fn new(open_jtalk: O) -> Self {
        Self(open_jtalk)
    }
}

impl<O: FullcontextExtractor> TextAnalyzer for OpenJTalkAnalyzer<O> {
    fn analyze(&self, text: &str) -> Result<Vec<AccentPhraseModel>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }
        Ok(extract_full_context_label(&self.0, text)?)
    }
}
