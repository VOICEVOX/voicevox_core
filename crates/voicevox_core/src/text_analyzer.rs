use crate::{
    engine::{self, parse_kana, MoraModel, Utterance},
    AccentPhraseModel, FullcontextExtractor, Result,
};

pub trait TextAnalyzer {
    fn analyze(&self, text: &str) -> Result<Vec<AccentPhraseModel>>;
}

/// AquesTalk風記法からAccentPhraseの配列を生成するTextAnalyzer
#[derive(Clone)]
pub struct KanaAnalyzer;

impl KanaAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
}

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
        let utterance = Utterance::extract_full_context_label(&self.0, text)?;
        Ok(utterance_to_accent_phrases(utterance))
    }
}

fn utterance_to_accent_phrases(utterance: Utterance) -> Vec<AccentPhraseModel> {
    let accent_phrases: Vec<AccentPhraseModel> = utterance.breath_groups().iter().enumerate().fold(
        Vec::new(),
        |mut accum_vec, (i, breath_group)| {
            accum_vec.extend(breath_group.accent_phrases().iter().enumerate().map(
                |(j, accent_phrase)| {
                    let moras = accent_phrase
                        .moras()
                        .iter()
                        .map(|mora| {
                            let mora_text = mora
                                .phonemes()
                                .iter()
                                .map(|phoneme| phoneme.phoneme().to_string())
                                .collect::<Vec<_>>()
                                .join("");

                            let (consonant, consonant_length) =
                                if let Some(consonant) = mora.consonant() {
                                    (Some(consonant.phoneme().to_string()), Some(0.))
                                } else {
                                    (None, None)
                                };

                            MoraModel::new(
                                mora_to_text(mora_text),
                                consonant,
                                consonant_length,
                                mora.vowel().phoneme().into(),
                                0.,
                                0.,
                            )
                        })
                        .collect();

                    let pause_mora = if i != utterance.breath_groups().len() - 1
                        && j == breath_group.accent_phrases().len() - 1
                    {
                        Some(MoraModel::new(
                            "、".into(),
                            None,
                            None,
                            "pau".into(),
                            0.,
                            0.,
                        ))
                    } else {
                        None
                    };

                    AccentPhraseModel::new(
                        moras,
                        *accent_phrase.accent(),
                        pause_mora,
                        *accent_phrase.is_interrogative(),
                    )
                },
            ));

            accum_vec
        },
    );

    accent_phrases
}

fn mora_to_text(mora: impl AsRef<str>) -> String {
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
