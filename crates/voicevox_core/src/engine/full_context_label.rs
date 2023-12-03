use std::collections::HashMap;

use crate::engine::open_jtalk::FullcontextExtractor;
use derive_getters::Getters;
use derive_new::new;
use once_cell::sync::Lazy;
use regex::Regex;

// FIXME: 入力テキストをここで持って、メッセージに含む
#[derive(thiserror::Error, Debug)]
#[error("入力テキストからのフルコンテキストラベル抽出に失敗しました: {context}")]
pub(crate) struct FullContextLabelError {
    context: ErrorKind,
    #[source]
    source: Option<anyhow::Error>,
}

#[derive(derive_more::Display, Debug)]
enum ErrorKind {
    #[display(fmt = "Open JTalkで解釈することができませんでした")]
    OpenJtalk,

    #[display(fmt = "label parse error label: {label}")]
    LabelParse { label: String },

    #[display(fmt = "too long mora mora_phonemes: {mora_phonemes:?}")]
    TooLongMora { mora_phonemes: Vec<Phoneme> },

    #[display(fmt = "invalid mora: {mora:?}")]
    InvalidMora { mora: Box<Mora> },
}

type Result<T> = std::result::Result<T, FullContextLabelError>;

#[derive(new, Getters, Clone, PartialEq, Eq, Debug)]
pub struct Phoneme {
    contexts: HashMap<String, String>,
    label: String,
}

static P3_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\-(.*?)\+)").unwrap());
static A2_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\+(\d+|xx)\+)").unwrap());
static A3_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\+(\d+|xx)/B:)").unwrap());
static F1_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(/F:(\d+|xx)_)").unwrap());
static F2_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(_(\d+|xx)\#)").unwrap());
static F3_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\#(\d+|xx)_)").unwrap());
static F5_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(@(\d+|xx)_)").unwrap());
static H1_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(/H:(\d+|xx)_)").unwrap());
static I3_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(@(\d+|xx)\+)").unwrap());
static J1_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(/J:(\d+|xx)_)").unwrap());

fn string_feature_by_regex(re: &Regex, label: &str) -> std::result::Result<String, ErrorKind> {
    if let Some(caps) = re.captures(label) {
        Ok(caps.get(2).unwrap().as_str().to_string())
    } else {
        Err(ErrorKind::LabelParse {
            label: label.into(),
        })
    }
}

impl Phoneme {
    fn from_label(label: impl Into<String>) -> std::result::Result<Self, ErrorKind> {
        let mut contexts = HashMap::<String, String>::with_capacity(10);
        let label = label.into();
        contexts.insert("p3".into(), string_feature_by_regex(&P3_REGEX, &label)?);
        contexts.insert("a2".into(), string_feature_by_regex(&A2_REGEX, &label)?);
        contexts.insert("a3".into(), string_feature_by_regex(&A3_REGEX, &label)?);
        contexts.insert("f1".into(), string_feature_by_regex(&F1_REGEX, &label)?);
        contexts.insert("f2".into(), string_feature_by_regex(&F2_REGEX, &label)?);
        contexts.insert("f3".into(), string_feature_by_regex(&F3_REGEX, &label)?);
        contexts.insert("f5".into(), string_feature_by_regex(&F5_REGEX, &label)?);
        contexts.insert("h1".into(), string_feature_by_regex(&H1_REGEX, &label)?);
        contexts.insert("i3".into(), string_feature_by_regex(&I3_REGEX, &label)?);
        contexts.insert("j1".into(), string_feature_by_regex(&J1_REGEX, &label)?);

        Ok(Self::new(contexts, label))
    }

    pub fn phoneme(&self) -> &str {
        self.contexts.get("p3").unwrap().as_str()
    }

    pub fn is_pause(&self) -> bool {
        self.contexts.get("f1").unwrap().as_str() == "xx"
    }
}

#[derive(new, Getters, Clone, PartialEq, Eq, Debug)]
pub struct Mora {
    consonant: Option<Phoneme>,
    vowel: Phoneme,
}

impl Mora {
    pub fn set_context(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        if let Some(ref mut consonant) = self.consonant {
            consonant.contexts.insert(key.clone(), value.clone());
        }
        self.vowel.contexts.insert(key, value);
    }

    pub fn phonemes(&self) -> Vec<Phoneme> {
        if self.consonant.is_some() {
            vec![
                self.consonant().as_ref().unwrap().clone(),
                self.vowel.clone(),
            ]
        } else {
            vec![self.vowel.clone()]
        }
    }

    #[allow(dead_code)]
    pub fn labels(&self) -> Vec<String> {
        self.phonemes().iter().map(|p| p.label().clone()).collect()
    }
}

#[derive(new, Getters, Clone, Debug, PartialEq, Eq)]
pub struct AccentPhrase {
    moras: Vec<Mora>,
    accent: usize,
    is_interrogative: bool,
}

impl AccentPhrase {
    fn from_phonemes(mut phonemes: Vec<Phoneme>) -> std::result::Result<Self, ErrorKind> {
        let mut moras = Vec::with_capacity(phonemes.len());
        let mut mora_phonemes = Vec::with_capacity(phonemes.len());
        for i in 0..phonemes.len() {
            {
                let phoneme = phonemes.get_mut(i).unwrap();
                if phoneme.contexts().get("a2").map(|s| s.as_str()) == Some("49") {
                    break;
                }
                mora_phonemes.push(phoneme.clone());
            }

            if i + 1 == phonemes.len()
                || phonemes.get(i).unwrap().contexts().get("a2").unwrap()
                    != phonemes.get(i + 1).unwrap().contexts().get("a2").unwrap()
            {
                if mora_phonemes.len() == 1 {
                    moras.push(Mora::new(None, mora_phonemes.get(0).unwrap().clone()));
                } else if mora_phonemes.len() == 2 {
                    moras.push(Mora::new(
                        Some(mora_phonemes.get(0).unwrap().clone()),
                        mora_phonemes.get(1).unwrap().clone(),
                    ));
                } else {
                    return Err(ErrorKind::TooLongMora { mora_phonemes });
                }
                mora_phonemes.clear();
            }
        }

        let mora = moras.get(0).unwrap();
        let mut accent: usize = mora
            .vowel()
            .contexts()
            .get("f2")
            .ok_or_else(|| ErrorKind::InvalidMora {
                mora: mora.clone().into(),
            })?
            .parse()
            .map_err(|_| ErrorKind::InvalidMora {
                mora: mora.clone().into(),
            })?;

        let is_interrogative = moras
            .last()
            .unwrap()
            .vowel()
            .contexts()
            .get("f3")
            .map(|s| s.as_str())
            == Some("1");
        // workaround for VOICEVOX/voicevox_engine#55
        if accent > moras.len() {
            accent = moras.len();
        }

        Ok(Self::new(moras, accent, is_interrogative))
    }

    #[allow(dead_code)]
    pub fn set_context(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        for mora in self.moras.iter_mut() {
            mora.set_context(&key, &value);
        }
    }

    pub fn phonemes(&self) -> Vec<Phoneme> {
        self.moras.iter().flat_map(|m| m.phonemes()).collect()
    }

    #[allow(dead_code)]
    pub fn labels(&self) -> Vec<String> {
        self.phonemes().iter().map(|p| p.label().clone()).collect()
    }

    #[allow(dead_code)]
    pub fn merge(&self, accent_phrase: AccentPhrase) -> AccentPhrase {
        let mut moras = self.moras().clone();
        let is_interrogative = *accent_phrase.is_interrogative();
        moras.extend(accent_phrase.moras);
        AccentPhrase::new(moras, *self.accent(), is_interrogative)
    }
}

#[derive(new, Getters, Clone, PartialEq, Eq, Debug)]
pub struct BreathGroup {
    accent_phrases: Vec<AccentPhrase>,
}

impl BreathGroup {
    fn from_phonemes(phonemes: Vec<Phoneme>) -> std::result::Result<Self, ErrorKind> {
        let mut accent_phrases = Vec::with_capacity(phonemes.len());
        let mut accent_phonemes = Vec::with_capacity(phonemes.len());
        for i in 0..phonemes.len() {
            accent_phonemes.push(phonemes.get(i).unwrap().clone());
            if i + 1 == phonemes.len()
                || phonemes.get(i).unwrap().contexts().get("i3").unwrap()
                    != phonemes.get(i + 1).unwrap().contexts().get("i3").unwrap()
                || phonemes.get(i).unwrap().contexts().get("f5").unwrap()
                    != phonemes.get(i + 1).unwrap().contexts().get("f5").unwrap()
            {
                accent_phrases.push(AccentPhrase::from_phonemes(accent_phonemes.clone())?);
                accent_phonemes.clear();
            }
        }

        Ok(Self::new(accent_phrases))
    }

    #[allow(dead_code)]
    pub fn set_context(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        for accent_phrase in self.accent_phrases.iter_mut() {
            accent_phrase.set_context(&key, &value);
        }
    }

    pub fn phonemes(&self) -> Vec<Phoneme> {
        self.accent_phrases()
            .iter()
            .flat_map(|a| a.phonemes())
            .collect()
    }

    #[allow(dead_code)]
    pub fn labels(&self) -> Vec<String> {
        self.phonemes().iter().map(|p| p.label().clone()).collect()
    }
}

#[derive(new, Getters, Clone, PartialEq, Eq, Debug)]
pub struct Utterance {
    breath_groups: Vec<BreathGroup>,
    pauses: Vec<Phoneme>,
}

impl Utterance {
    fn from_phonemes(phonemes: Vec<Phoneme>) -> std::result::Result<Self, ErrorKind> {
        let mut breath_groups = vec![];
        let mut group_phonemes = Vec::with_capacity(phonemes.len());
        let mut pauses = vec![];
        for phoneme in phonemes.into_iter() {
            if !phoneme.is_pause() {
                group_phonemes.push(phoneme);
            } else {
                pauses.push(phoneme);

                if !group_phonemes.is_empty() {
                    breath_groups.push(BreathGroup::from_phonemes(group_phonemes.clone())?);
                    group_phonemes.clear();
                }
            }
        }
        Ok(Self::new(breath_groups, pauses))
    }

    #[allow(dead_code)]
    pub fn set_context(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        for breath_group in self.breath_groups.iter_mut() {
            breath_group.set_context(&key, &value);
        }
    }

    #[allow(dead_code)]
    pub fn phonemes(&self) -> Vec<Phoneme> {
        // TODO:実装が中途半端なのであとでちゃんと実装する必要があるらしい
        // https://github.com/VOICEVOX/voicevox_core/pull/174#discussion_r919982651
        let mut phonemes = Vec::with_capacity(self.breath_groups.len());

        for i in 0..self.pauses().len() {
            phonemes.push(self.pauses().get(i).unwrap().clone());
            if i < self.pauses().len() - 1 {
                let p = self.breath_groups().get(i).unwrap().phonemes();
                phonemes.extend(p);
            }
        }
        phonemes
    }

    #[allow(dead_code)]
    pub fn labels(&self) -> Vec<String> {
        self.phonemes().iter().map(|p| p.label().clone()).collect()
    }

    pub(crate) fn extract_full_context_label(
        open_jtalk: &impl FullcontextExtractor,
        text: impl AsRef<str>,
    ) -> Result<Self> {
        let labels = open_jtalk
            .extract_fullcontext(text.as_ref())
            .map_err(|source| FullContextLabelError {
                context: ErrorKind::OpenJtalk,
                source: Some(source),
            })?;

        labels
            .into_iter()
            .map(Phoneme::from_label)
            .collect::<std::result::Result<Vec<_>, _>>()
            .and_then(Self::from_phonemes)
            .map_err(|context| FullContextLabelError {
                context,
                source: None,
            })
    }
}
