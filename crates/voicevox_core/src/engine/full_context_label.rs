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
        Ok(caps[2].to_string())
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
                    moras.push(Mora::new(None, mora_phonemes[0].clone()));
                } else if mora_phonemes.len() == 2 {
                    moras.push(Mora::new(
                        Some(mora_phonemes[0].clone()),
                        mora_phonemes[1].clone(),
                    ));
                } else {
                    return Err(ErrorKind::TooLongMora { mora_phonemes });
                }
                mora_phonemes.clear();
            }
        }

        let mora = &moras[0];
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

#[cfg(test)]
mod tests {
    use rstest_reuse::*;

    use ::test_util::OPEN_JTALK_DIC_DIR;
    use rstest::rstest;

    use crate::{
        engine::{open_jtalk::FullcontextExtractor, MoraModel},
        text_analyzer::{OpenJTalkAnalyzer, TextAnalyzer},
        AccentPhraseModel,
    };

    fn mora_model(text: &str, consonant: Option<&str>, vowel: &str) -> MoraModel {
        MoraModel::new(
            text.into(),
            consonant.map(|c| c.into()),
            consonant.and(Some(0.0)),
            vowel.into(),
            0.0,
            0.0,
        )
    }

    #[template]
    #[rstest]
    #[case(
        "いぇ",
        &[
            "xx^xx-sil+y=e/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:1_1%0_xx_xx/H:xx_xx/I:xx-xx@xx+xx&xx-xx|xx+xx/J:1_1/K:1+1-1",
            "xx^sil-y+e=sil/A:0+1+1/B:xx-xx_xx/C:09_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:1_1#0_xx@1_1|1_1/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-1@1+1&1-1|1+1/J:xx_xx/K:1+1-1",
            "sil^y-e+sil=xx/A:0+1+1/B:xx-xx_xx/C:09_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:1_1#0_xx@1_1|1_1/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-1@1+1&1-1|1+1/J:xx_xx/K:1+1-1",
            "y^e-sil+xx=xx/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:1_1!0_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:xx_xx%xx_xx_xx/H:1_1/I:xx-xx@xx+xx&xx-xx|xx+xx/J:xx_xx/K:1+1-1",
        ],
        &[
            AccentPhraseModel::new(
                vec![mora_model("イェ", Some("y"), "e")],
                1,
                None,
                false,
            )
        ]
    )]
    #[case(
        "んーっ",
        &[
            "xx^xx-sil+N=N/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:3_3%0_xx_xx/H:xx_xx/I:xx-xx@xx+xx&xx-xx|xx+xx/J:1_3/K:1+1-3",
            "xx^sil-N+N=cl/A:-2+1+3/B:xx-xx_xx/C:09_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_1|1_3/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-3@1+1&1-1|1+3/J:xx_xx/K:1+1-3",
            "sil^N-N+cl=sil/A:-1+2+2/B:xx-xx_xx/C:09_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_1|1_3/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-3@1+1&1-1|1+3/J:xx_xx/K:1+1-3",
            "N^N-cl+sil=xx/A:0+3+1/B:09-xx_xx/C:09_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:3_3#0_xx@1_1|1_3/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-3@1+1&1-1|1+3/J:xx_xx/K:1+1-3",
            "N^cl-sil+xx=xx/A:xx+xx+xx/B:09-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:3_3!0_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:xx_xx%xx_xx_xx/H:1_3/I:xx-xx@xx+xx&xx-xx|xx+xx/J:xx_xx/K:1+1-3",
        ],
        &[
            AccentPhraseModel::new(
                vec![
                    mora_model("ン", None, "N"),
                    mora_model("ン", None, "N"),
                    mora_model("ッ", None, "cl"),
                ],
                3,
                None,
                false,
            ),
        ]
    )]
    #[case(
        "これはテストです",
        &[
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
        ],
        &[
            AccentPhraseModel::new(
                vec![
                    mora_model("コ", Some("k"), "o"),
                    mora_model("レ", Some("r"), "e"),
                    mora_model("ワ", Some("w"), "a"),
                ],
                3,
                None,
                false,
            ),
            AccentPhraseModel::new(
                vec![
                    mora_model("テ", Some("t"), "e"),
                    mora_model("ス", Some("s"), "U"),
                    mora_model("ト", Some("t"), "o"),
                    mora_model("デ", Some("d"), "e"),
                    mora_model("ス", Some("s"), "U"),
                ],
                1,
                None,
                false,
            ),
        ]
    )]
    #[case(
        "１、１０００、１００万、１億？",
        &[
            "xx^xx-sil+i=ch/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:05+xx_xx/E:xx_xx!xx_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:2_2%0_xx_xx/H:xx_xx/I:xx-xx@xx+xx&xx-xx|xx+xx/J:1_2/K:4+4-12",
            "xx^sil-i+ch=i/A:-1+1+2/B:xx-xx_xx/C:05_xx+xx/D:05+xx_xx/E:xx_xx!xx_xx-xx/F:2_2#0_xx@1_1|1_2/G:2_1%0_xx_0/H:xx_xx/I:1-2@1+4&1-4|1+12/J:1_2/K:4+4-12",
            "sil^i-ch+i=pau/A:0+2+1/B:xx-xx_xx/C:05_xx+xx/D:05+xx_xx/E:xx_xx!xx_xx-xx/F:2_2#0_xx@1_1|1_2/G:2_1%0_xx_0/H:xx_xx/I:1-2@1+4&1-4|1+12/J:1_2/K:4+4-12",
            "i^ch-i+pau=s/A:0+2+1/B:xx-xx_xx/C:05_xx+xx/D:05+xx_xx/E:xx_xx!xx_xx-xx/F:2_2#0_xx@1_1|1_2/G:2_1%0_xx_0/H:xx_xx/I:1-2@1+4&1-4|1+12/J:1_2/K:4+4-12",
            "ch^i-pau+s=e/A:xx+xx+xx/B:05-xx_xx/C:xx_xx+xx/D:05+xx_xx/E:2_2!0_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:2_1%0_xx_xx/H:1_2/I:xx-xx@xx+xx&xx-xx|xx+xx/J:1_2/K:4+4-12",
            "i^pau-s+e=N/A:0+1+2/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:2_2!0_xx-0/F:2_1#0_xx@1_1|1_2/G:4_3%0_xx_0/H:1_2/I:1-2@2+3&2-3|3+10/J:1_4/K:4+4-12",
            "pau^s-e+N=pau/A:0+1+2/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:2_2!0_xx-0/F:2_1#0_xx@1_1|1_2/G:4_3%0_xx_0/H:1_2/I:1-2@2+3&2-3|3+10/J:1_4/K:4+4-12",
            "s^e-N+pau=hy/A:1+2+1/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:2_2!0_xx-0/F:2_1#0_xx@1_1|1_2/G:4_3%0_xx_0/H:1_2/I:1-2@2+3&2-3|3+10/J:1_4/K:4+4-12",
            "e^N-pau+hy=a/A:xx+xx+xx/B:05-xx_xx/C:xx_xx+xx/D:05+xx_xx/E:2_1!0_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:4_3%0_xx_xx/H:1_2/I:xx-xx@xx+xx&xx-xx|xx+xx/J:1_4/K:4+4-12",
            "N^pau-hy+a=k/A:-2+1+4/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:2_1!0_xx-0/F:4_3#0_xx@1_1|1_4/G:4_2%1_xx_0/H:1_2/I:1-4@3+2&3-2|5+8/J:1_4/K:4+4-12",
            "pau^hy-a+k=u/A:-2+1+4/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:2_1!0_xx-0/F:4_3#0_xx@1_1|1_4/G:4_2%1_xx_0/H:1_2/I:1-4@3+2&3-2|5+8/J:1_4/K:4+4-12",
            "hy^a-k+u=m/A:-1+2+3/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:2_1!0_xx-0/F:4_3#0_xx@1_1|1_4/G:4_2%1_xx_0/H:1_2/I:1-4@3+2&3-2|5+8/J:1_4/K:4+4-12",
            "a^k-u+m=a/A:-1+2+3/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:2_1!0_xx-0/F:4_3#0_xx@1_1|1_4/G:4_2%1_xx_0/H:1_2/I:1-4@3+2&3-2|5+8/J:1_4/K:4+4-12",
            "k^u-m+a=N/A:0+3+2/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:2_1!0_xx-0/F:4_3#0_xx@1_1|1_4/G:4_2%1_xx_0/H:1_2/I:1-4@3+2&3-2|5+8/J:1_4/K:4+4-12",
            "u^m-a+N=pau/A:0+3+2/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:2_1!0_xx-0/F:4_3#0_xx@1_1|1_4/G:4_2%1_xx_0/H:1_2/I:1-4@3+2&3-2|5+8/J:1_4/K:4+4-12",
            "m^a-N+pau=i/A:1+4+1/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:2_1!0_xx-0/F:4_3#0_xx@1_1|1_4/G:4_2%1_xx_0/H:1_2/I:1-4@3+2&3-2|5+8/J:1_4/K:4+4-12",
            "a^N-pau+i=ch/A:xx+xx+xx/B:05-xx_xx/C:xx_xx+xx/D:05+xx_xx/E:4_3!0_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:4_2%1_xx_xx/H:1_4/I:xx-xx@xx+xx&xx-xx|xx+xx/J:1_4/K:4+4-12",
            "N^pau-i+ch=i/A:-1+1+4/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:4_3!0_xx-0/F:4_2#1_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_4/I:1-4@4+1&4-1|9+4/J:xx_xx/K:4+4-12",
            "pau^i-ch+i=o/A:0+2+3/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:4_3!0_xx-0/F:4_2#1_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_4/I:1-4@4+1&4-1|9+4/J:xx_xx/K:4+4-12",
            "i^ch-i+o=k/A:0+2+3/B:05-xx_xx/C:05_xx+xx/D:05+xx_xx/E:4_3!0_xx-0/F:4_2#1_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_4/I:1-4@4+1&4-1|9+4/J:xx_xx/K:4+4-12",
            "ch^i-o+k=u/A:1+3+2/B:05-xx_xx/C:05_xx+xx/D:xx+xx_xx/E:4_3!0_xx-0/F:4_2#1_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_4/I:1-4@4+1&4-1|9+4/J:xx_xx/K:4+4-12",
            "i^o-k+u=sil/A:2+4+1/B:05-xx_xx/C:05_xx+xx/D:xx+xx_xx/E:4_3!0_xx-0/F:4_2#1_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_4/I:1-4@4+1&4-1|9+4/J:xx_xx/K:4+4-12",
            "o^k-u+sil=xx/A:2+4+1/B:05-xx_xx/C:05_xx+xx/D:xx+xx_xx/E:4_3!0_xx-0/F:4_2#1_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_4/I:1-4@4+1&4-1|9+4/J:xx_xx/K:4+4-12",
            "k^u-sil+xx=xx/A:xx+xx+xx/B:05-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:4_2!1_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:xx_xx%xx_xx_xx/H:1_4/I:xx-xx@xx+xx&xx-xx|xx+xx/J:xx_xx/K:4+4-12",
        ],
        &[
            AccentPhraseModel::new(
                vec![
                    mora_model("イ", None, "i"),
                    mora_model("チ", Some("ch"), "i"),
                ],
                2,
                Some(mora_model("、", None, "pau")),
                false,
            ),
            AccentPhraseModel::new(
                vec![
                    mora_model("セ", Some("s"), "e"),
                    mora_model("ン", None, "N"),
                ],
                1,
                Some(mora_model("、", None, "pau")),
                false,
            ),
            AccentPhraseModel::new(
                vec![
                    mora_model("ヒャ", Some("hy"), "a"),
                    mora_model("ク", Some("k"), "u"),
                    mora_model("マ", Some("m"), "a"),
                    mora_model("ン", None, "N"),
                ],
                3,
                Some(mora_model("、", None, "pau")),
                false,
            ),
            AccentPhraseModel::new(
                vec![
                    mora_model("イ", None, "i"),
                    mora_model("チ", Some("ch"), "i"),
                    mora_model("オ", None, "o"),
                    mora_model("ク", Some("k"), "u"),
                ],
                2,
                None,
                true,
            ),
        ]
    )]
    #[case(
        "クヮルテット。あーあ、。",
        &[
            "xx^xx-sil+kw=a/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:02+xx_xx/E:xx_xx!xx_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:5_3%0_xx_xx/H:xx_xx/I:xx-xx@xx+xx&xx-xx|xx+xx/J:1_5/K:2+3-8",
            "xx^sil-kw+a=r/A:-2+1+5/B:xx-xx_xx/C:02_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx/F:5_3#0_xx@1_1|1_5/G:2_1%0_xx_0/H:xx_xx/I:1-5@1+2&1-3|1+8/J:2_3/K:2+3-8",
            "sil^kw-a+r=u/A:-2+1+5/B:xx-xx_xx/C:02_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx/F:5_3#0_xx@1_1|1_5/G:2_1%0_xx_0/H:xx_xx/I:1-5@1+2&1-3|1+8/J:2_3/K:2+3-8",
            "kw^a-r+u=t/A:-1+2+4/B:xx-xx_xx/C:02_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx/F:5_3#0_xx@1_1|1_5/G:2_1%0_xx_0/H:xx_xx/I:1-5@1+2&1-3|1+8/J:2_3/K:2+3-8",
            "a^r-u+t=e/A:-1+2+4/B:xx-xx_xx/C:02_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx/F:5_3#0_xx@1_1|1_5/G:2_1%0_xx_0/H:xx_xx/I:1-5@1+2&1-3|1+8/J:2_3/K:2+3-8",
            "r^u-t+e=cl/A:0+3+3/B:xx-xx_xx/C:02_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx/F:5_3#0_xx@1_1|1_5/G:2_1%0_xx_0/H:xx_xx/I:1-5@1+2&1-3|1+8/J:2_3/K:2+3-8",
            "u^t-e+cl=t/A:0+3+3/B:xx-xx_xx/C:02_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx/F:5_3#0_xx@1_1|1_5/G:2_1%0_xx_0/H:xx_xx/I:1-5@1+2&1-3|1+8/J:2_3/K:2+3-8",
            "t^e-cl+t=o/A:1+4+2/B:xx-xx_xx/C:02_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx/F:5_3#0_xx@1_1|1_5/G:2_1%0_xx_0/H:xx_xx/I:1-5@1+2&1-3|1+8/J:2_3/K:2+3-8",
            "e^cl-t+o=pau/A:2+5+1/B:xx-xx_xx/C:02_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx/F:5_3#0_xx@1_1|1_5/G:2_1%0_xx_0/H:xx_xx/I:1-5@1+2&1-3|1+8/J:2_3/K:2+3-8",
            "cl^t-o+pau=a/A:2+5+1/B:xx-xx_xx/C:02_xx+xx/D:09+xx_xx/E:xx_xx!xx_xx-xx/F:5_3#0_xx@1_1|1_5/G:2_1%0_xx_0/H:xx_xx/I:1-5@1+2&1-3|1+8/J:2_3/K:2+3-8",
            "t^o-pau+a=a/A:xx+xx+xx/B:02-xx_xx/C:xx_xx+xx/D:09+xx_xx/E:5_3!0_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:2_1%0_xx_xx/H:1_5/I:xx-xx@xx+xx&xx-xx|xx+xx/J:2_3/K:2+3-8",
            "o^pau-a+a=a/A:0+1+2/B:02-xx_xx/C:09_xx+xx/D:09+xx_xx/E:5_3!0_xx-0/F:2_1#0_xx@1_2|1_3/G:1_1%0_xx_1/H:1_5/I:2-3@2+1&2-2|6+3/J:xx_xx/K:2+3-8",
            "pau^a-a+a=sil/A:1+2+1/B:02-xx_xx/C:09_xx+xx/D:09+xx_xx/E:5_3!0_xx-0/F:2_1#0_xx@1_2|1_3/G:1_1%0_xx_1/H:1_5/I:2-3@2+1&2-2|6+3/J:xx_xx/K:2+3-8",
            "a^a-a+sil=xx/A:0+1+1/B:09-xx_xx/C:09_xx+xx/D:xx+xx_xx/E:2_1!0_xx-1/F:1_1#0_xx@2_1|3_1/G:xx_xx%xx_xx_xx/H:1_5/I:2-3@2+1&2-2|6+3/J:xx_xx/K:2+3-8",
            "a^a-sil+xx=xx/A:xx+xx+xx/B:09-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:1_1!0_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:xx_xx%xx_xx_xx/H:2_3/I:xx-xx@xx+xx&xx-xx|xx+xx/J:xx_xx/K:2+3-8",
        ],
        &[
            AccentPhraseModel::new(
                vec![
                    mora_model("クヮ", Some("kw"), "a"),
                    mora_model("ル", Some("r"), "u"),
                    mora_model("テ", Some("t"), "e"),
                    mora_model("ッ", None, "cl"),
                    mora_model("ト", Some("t"), "o"),
                ],
                3,
                Some(mora_model("、", None, "pau")),
                false,
            ),
            AccentPhraseModel::new(
                vec![
                    mora_model("ア", None, "a"),
                    mora_model("ア", None, "a"),
                ],
                1,
                None,
                false,
            ),
            AccentPhraseModel::new(
                vec![mora_model("ア", None, "a")],
                1,
                None,
                false,
            ),
        ]
    )]
    fn label_cases(
        #[case] text: &str,
        #[case] labels: &[&str],
        #[case] accent_phrase: &[AccentPhraseModel],
    ) {
    }

    #[apply(label_cases)]
    #[tokio::test]
    async fn open_jtalk(text: &str, labels: &[&str], _accent_phrase: &[AccentPhraseModel]) {
        let open_jtalk = crate::tokio::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
            .await
            .unwrap();
        assert_eq!(&open_jtalk.extract_fullcontext(text).unwrap(), labels);
    }

    #[apply(label_cases)]
    #[tokio::test]
    async fn extract_fullcontext(
        text: &str,
        _labels: &[&str],
        accent_phrase: &[AccentPhraseModel],
    ) {
        let open_jtalk = crate::tokio::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
            .await
            .unwrap();
        let analyzer = OpenJTalkAnalyzer::new(open_jtalk);
        assert_eq!(analyzer.analyze(text).unwrap(), accent_phrase);
    }
}
