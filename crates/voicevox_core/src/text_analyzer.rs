use crate::{
    engine::{self, extract_full_context_label, parse_kana},
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

pub fn mora_to_text(mora: impl AsRef<str>) -> String {
    let last_char = mora.as_ref().chars().last().unwrap();
    let mora = if ['A', 'I', 'U', 'E', 'O'].contains(&last_char) {
        format!(
            "{}{}",
            &mora.as_ref()[0..mora.as_ref().len() - 1],
            last_char.to_lowercase()
        )
    } else {
        mora.as_ref().to_string()
    };
    // もしカタカナに変換できなければ、引数で与えた文字列がそのまま返ってくる
    engine::mora2text(&mora).to_string()
}
