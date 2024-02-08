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

    generate_accentphrases(parsed_labels).map_err(|context| FullContextLabelError {
        context,
        source: None,
    })
}

fn generate_accentphrases(
    utterance: Vec<Label>,
) -> std::result::Result<Vec<AccentPhraseModel>, ErrorKind> {
    SplitGroupByKey::new(&utterance, |label| {
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
    })
    .filter_map(|labels| {
        let moras = match generate_moras(labels) {
            Ok(moras) => moras,
            Err(err) => return Some(Err(err)),
        };

        let Some(label) = labels.first() else {
            return None;
        };
        let Some(ap_curr) = label.accent_phrase_curr.as_ref() else {
            return None;
        };
        let Some(bg_curr) = label.breath_group_curr.as_ref() else {
            return None;
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
        let mut accent = ap_curr.accent_position as usize;
        if accent > moras.len() {
            accent = moras.len();
        }

        Some(Ok(AccentPhraseModel::new(
            moras,
            accent,
            pause_mora,
            ap_curr.is_interrogative,
        )))
    })
    .collect::<std::result::Result<Vec<_>, _>>()
}

fn generate_moras(labels: &[Label]) -> std::result::Result<Vec<MoraModel>, ErrorKind> {
    SplitGroupByKey::new(labels, |label| {
        label.mora.as_ref().map(|mora| mora.position_forward)
    })
    .filter_map(|labels| {
        let mut label_iter = labels.iter().filter(|label| label.mora.is_some());
        let mora_model = match (label_iter.next(), label_iter.next(), label_iter.next()) {
            (Some(consonant), Some(vowel), None) => generate_mora(Some(consonant), vowel),
            (Some(vowel), None, None) => generate_mora(None, vowel),

            // silやpau以外の音素がないモーラは含めない
            (None, _, _) => return None,
            // 音素が3つ以上あるとき
            (Some(first), _, Some(_)) => {
                if first.mora.as_ref().map(|mora| mora.position_forward) == Some(49) {
                    // position_forwardが飽和している場合は正常として扱う
                    return None;
                } else {
                    return Some(Err(ErrorKind::TooLongMora));
                }
            }
        };
        Some(Ok(mora_model))
    })
    .collect::<std::result::Result<Vec<_>, _>>()
}

fn generate_mora(consonant: Option<&Label>, vowel: &Label) -> MoraModel {
    let consonant_phoneme = consonant.and_then(|c| c.phoneme.c.to_owned());
    let vowel_phoneme = vowel.phoneme.c.as_deref().unwrap();
    let mora_text = format!(
        "{}{}",
        consonant_phoneme.as_deref().unwrap_or(""),
        match vowel_phoneme {
            phoneme @ ("A" | "I" | "U" | "E" | "O") => phoneme.to_lowercase(),
            phoneme => phoneme.to_string(),
        }
    );
    MoraModel::new(
        engine::mora2text(&mora_text).to_string(),
        consonant_phoneme,
        consonant.and(Some(0.0)),
        vowel_phoneme.to_string(),
        0.0,
        0.0,
    )
}

#[derive(new)]
struct SplitGroupByKey<'a, T, F, V>
where
    F: FnMut(&T) -> V,
    V: Eq,
{
    array: &'a [T],
    func: F,
}

impl<'a, T, F, V> Iterator for SplitGroupByKey<'a, T, F, V>
where
    F: FnMut(&T) -> V,
    V: Eq,
{
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        if self.array.is_empty() {
            return None;
        }

        let mut index = 0;
        let mut current_v = None;
        while let Some(item) = self.array.get(index) {
            let v = Some((self.func)(item));
            if current_v.is_some() && current_v != v {
                break;
            }
            current_v = v;
            index += 1;
        }
        let (result, rest) = self.array.split_at(index);
        self.array = rest;
        Some(result)
    }
}
