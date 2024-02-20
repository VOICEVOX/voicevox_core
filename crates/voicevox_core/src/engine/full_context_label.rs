use std::str::FromStr;

use crate::{
    engine::{self, open_jtalk::FullcontextExtractor, MoraModel},
    AccentPhraseModel,
};
use jlabel::Label;
use smallvec::SmallVec;

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
            source: Some(source.into()),
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
            .map(|label| label.utterance.accent_phrase_count.into())
            .unwrap_or(0),
    );

    let split = utterance.chunk_by(|a, b| {
        a.breath_group_curr == b.breath_group_curr && a.accent_phrase_curr == b.accent_phrase_curr
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
        let accent = usize::from(ap_curr.accent_position).min(moras.len());

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

    let split = accent_phrase.chunk_by(|a, b| a.mora == b.mora);
    for labels in split {
        let labels: SmallVec<[&Label; 3]> =
            labels.iter().filter(|label| label.mora.is_some()).collect();
        match labels[..] {
            [consonant, vowel] => {
                let mora = generate_mora(Some(consonant), vowel);
                moras.push(mora);
            }
            [vowel] => {
                let mora = generate_mora(None, vowel);
                moras.push(mora);
            }
            // silやpau以外の音素がないモーラは含めない
            [] => {}
            // 音素が3つ以上あるとき
            [first, ..] => {
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
        consonant.unwrap_or(""),
        match vowel {
            phoneme @ ("A" | "I" | "U" | "E" | "O") => phoneme.to_lowercase(),
            phoneme => phoneme.to_string(),
        }
    );
    // もしカタカナに変換できなければ、引数で与えた文字列がそのまま返ってくる
    engine::mora2text(&mora_text).to_string()
}

// FIXME: Remove `chunk_by` module after Rust 1.77.0 is released as stable.
use chunk_by::*;
mod chunk_by {
    // Implementations in this module were copied from
    // [Rust](https://github.com/rust-lang/rust/blob/746a58d4359786e4aebb372a30829706fa5a968f/library/core/src/slice/iter.rs).

    // MIT License Notice

    // Permission is hereby granted, free of charge, to any
    // person obtaining a copy of this software and associated
    // documentation files (the "Software"), to deal in the
    // Software without restriction, including without
    // limitation the rights to use, copy, modify, merge,
    // publish, distribute, sublicense, and/or sell copies of
    // the Software, and to permit persons to whom the Software
    // is furnished to do so, subject to the following
    // conditions:
    //
    // The above copyright notice and this permission notice
    // shall be included in all copies or substantial portions
    // of the Software.
    //
    // THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
    // ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
    // TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
    // PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
    // SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
    // CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
    // OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
    // IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    // DEALINGS IN THE SOFTWARE.

    pub struct ChunkBy<'a, T, P> {
        slice: &'a [T],
        predicate: P,
    }
    impl<'a, T, P> ChunkBy<'a, T, P> {
        pub(super) fn new(slice: &'a [T], predicate: P) -> Self {
            ChunkBy { slice, predicate }
        }
    }
    impl<'a, T, P> Iterator for ChunkBy<'a, T, P>
    where
        P: FnMut(&T, &T) -> bool,
    {
        type Item = &'a [T];

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.slice.is_empty() {
                None
            } else {
                let mut len = 1;
                let mut iter = self.slice.windows(2);
                while let Some([l, r]) = iter.next() {
                    if (self.predicate)(l, r) {
                        len += 1
                    } else {
                        break;
                    }
                }
                let (head, tail) = self.slice.split_at(len);
                self.slice = tail;
                Some(head)
            }
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            if self.slice.is_empty() {
                (0, Some(0))
            } else {
                (1, Some(self.slice.len()))
            }
        }
    }

    #[easy_ext::ext(TChunkBy)]
    impl<T> [T] {
        pub fn chunk_by<F>(&self, pred: F) -> ChunkBy<'_, T, F>
        where
            F: FnMut(&T, &T) -> bool,
        {
            ChunkBy::new(self, pred)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::TChunkBy;

        #[test]
        fn chunk_by() {
            let mut split = [0, 0, 1, 1, 1, -5].chunk_by(|a, b| a == b);
            assert_eq!(split.next(), Some([0, 0].as_slice()));
            assert_eq!(split.next(), Some([1, 1, 1].as_slice()));
            assert_eq!(split.next(), Some([-5].as_slice()));
            assert_eq!(split.next(), None);
        }
    }
}
