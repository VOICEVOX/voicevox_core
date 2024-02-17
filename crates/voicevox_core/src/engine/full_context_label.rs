use std::str::FromStr;

use crate::{
    engine::{self, open_jtalk::FullcontextExtractor, MoraModel},
    AccentPhraseModel,
};
use derive_new::new;
use jlabel::Label;

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

    #[display(fmt = "jlabelでラベルを解釈することができませんでした")]
    Jlabel,

    #[display(fmt = "too long mora")]
    TooLongMora,
}

type Result<T> = std::result::Result<T, FullContextLabelError>;

pub(crate) fn extract_full_context_label(
    open_jtalk: &impl FullcontextExtractor,
    text: impl AsRef<str>,
) -> Result<Vec<AccentPhraseModel>> {
    let labels = open_jtalk
        .extract_fullcontext(text.as_ref())
        .map_err(|source| FullContextLabelError {
            context: ErrorKind::OpenJtalk,
            source: Some(source),
        })?;

    let parsed_labels = labels
        .into_iter()
        .map(|s| Label::from_str(&s))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|source| FullContextLabelError {
            context: ErrorKind::Jlabel,
            source: Some(anyhow::anyhow!("{}", source)),
        })?;

    generate_accent_phrases(&parsed_labels).map_err(|context| FullContextLabelError {
        context,
        source: None,
    })
}

fn generate_accent_phrases(
    utterance: &[Label],
) -> std::result::Result<Vec<AccentPhraseModel>, ErrorKind> {
    let mut accent_phrases = Vec::with_capacity(
        utterance
            .first()
            .map(|label| label.utterance.accent_phrase_count as usize)
            .unwrap_or(0),
    );

    let split = SplitByKey::new(utterance, |label| {
        (
            label
                .breath_group_curr
                .as_ref()
                .map(|bg| bg.breath_group_position_backward),
            label
                .accent_phrase_curr
                .as_ref()
                .map(|ap| ap.accent_phrase_position_forward),
        )
    });
    for labels in split {
        let moras = generate_moras(labels)?;
        if moras.is_empty() {
            continue;
        }

        let Some(Label {
            accent_phrase_curr: Some(ap_curr),
            breath_group_curr: Some(bg_curr),
            ..
        }) = labels.first()
        else {
            continue;
        };

        // Breath Groupの中で最後のアクセント句かつ，Utteranceの中で最後のBreath Groupでない場合は次がpauになる
        let pause_mora = if ap_curr.accent_phrase_position_backward == 1
            && bg_curr.breath_group_position_backward != 1
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

        // workaround for VOICEVOX/voicevox_engine#55
        let accent = (ap_curr.accent_position as usize).min(moras.len());

        accent_phrases.push(AccentPhraseModel::new(
            moras,
            accent,
            pause_mora,
            ap_curr.is_interrogative,
        ))
    }
    Ok(accent_phrases)
}

fn generate_moras(accent_phrase: &[Label]) -> std::result::Result<Vec<MoraModel>, ErrorKind> {
    let mut moras = Vec::with_capacity(accent_phrase.len());

    let split = SplitByKey::new(accent_phrase, |label| {
        label.mora.as_ref().map(|mora| mora.position_forward)
    });
    for labels in split {
        let mut label_iter = labels.iter().filter(|label| label.mora.is_some());
        match (label_iter.next(), label_iter.next(), label_iter.next()) {
            (Some(consonant), Some(vowel), None) => {
                let mora = generate_mora(Some(consonant), vowel);
                moras.push(mora);
            }
            (Some(vowel), None, None) => {
                let mora = generate_mora(None, vowel);
                moras.push(mora);
            }
            // silやpau以外の音素がないモーラは含めない
            (None, _, _) => {}
            // 音素が3つ以上あるとき
            (Some(first), _, Some(_)) => {
                // position_forwardが飽和している場合は正常として扱う
                if first.mora.as_ref().map(|mora| mora.position_forward) != Some(49) {
                    return Err(ErrorKind::TooLongMora);
                }
            }
        }
    }
    Ok(moras)
}

fn generate_mora(consonant: Option<&Label>, vowel: &Label) -> MoraModel {
    let consonant_phoneme = consonant.and_then(|c| c.phoneme.c.to_owned());
    let vowel_phoneme = vowel.phoneme.c.as_deref().unwrap();
    MoraModel::new(
        mora_to_text(consonant_phoneme.as_deref(), vowel_phoneme),
        consonant_phoneme,
        consonant.and(Some(0.0)),
        vowel_phoneme.to_string(),
        0.0,
        0.0,
    )
}

pub fn mora_to_text(consonant: Option<&str>, vowel: &str) -> String {
    let mora_text = format!(
        "{}{}",
        consonant.as_deref().unwrap_or(""),
        match vowel {
            phoneme @ ("A" | "I" | "U" | "E" | "O") => phoneme.to_lowercase(),
            phoneme => phoneme.to_string(),
        }
    );
    // もしカタカナに変換できなければ、引数で与えた文字列がそのまま返ってくる
    engine::mora2text(&mora_text).to_string()
}

#[derive(new)]
struct SplitByKey<'a, T, F, V>
where
    F: FnMut(&T) -> V,
    V: Eq,
{
    array: &'a [T],
    pred: F,
}

impl<'a, T, F, V> Iterator for SplitByKey<'a, T, F, V>
where
    F: FnMut(&T) -> V,
    V: Eq,
{
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        let (first, rest) = self.array.split_first()?;
        let v = (self.pred)(first);
        let pos = rest
            .iter()
            .position(|x| (self.pred)(x) != v)
            .map_or(self.array.len(), |x| x + 1); // translate to index of `self.array`

        let (result, rest) = self.array.split_at(pos);
        self.array = rest;
        Some(result)
    }
}
