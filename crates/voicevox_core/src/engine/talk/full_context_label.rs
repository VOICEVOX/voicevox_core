use std::str::FromStr;

use jlabel::Label;
use smallvec::SmallVec;

use crate::AccentPhrase;

use super::{super::mora_mappings::MORA_PHONEMES_TO_MORA_KANA, open_jtalk::FullcontextExtractor};

#[derive(thiserror::Error, Debug)]
#[error("入力テキストからのフルコンテキストラベル抽出に失敗しました: {context}")]
pub(crate) struct FullContextLabelError {
    context: ErrorKind,
    #[source]
    source: Option<anyhow::Error>,
}

#[derive(derive_more::Display, Debug)]
enum ErrorKind {
    #[display("Open JTalkで解釈することができませんでした")]
    OpenJtalk,

    #[display("jlabelでラベルを解釈することができませんでした")]
    Jlabel,

    #[display("too long mora")]
    TooLongMora,
}

type Result<T> = std::result::Result<T, FullContextLabelError>;

pub(crate) fn extract_full_context_label(
    open_jtalk: &impl FullcontextExtractor,
    text: impl AsRef<str>,
) -> Result<Vec<AccentPhrase>> {
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
) -> std::result::Result<Vec<AccentPhrase>, ErrorKind> {
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
            Some(crate::Mora {
                text: "、".into(),
                consonant: None,
                consonant_length: None,
                vowel: "pau".into(),
                vowel_length: 0.,
                pitch: 0.,
            })
        } else {
            None
        };

        // workaround for VOICEVOX/voicevox_engine#55
        let accent = usize::from(ap_curr.accent_position).min(moras.len());

        accent_phrases.push(AccentPhrase {
            moras,
            accent,
            pause_mora,
            is_interrogative: ap_curr.is_interrogative,
        })
    }
    Ok(accent_phrases)
}

fn generate_moras(accent_phrase: &[Label]) -> std::result::Result<Vec<crate::Mora>, ErrorKind> {
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

            // 音素が3つ以上ある場合：
            // position_forwardとposition_backwardが飽和している場合は無視する
            [Label {
                mora:
                    Some(jlabel::Mora {
                        position_forward: 49,
                        position_backward: 49,
                        ..
                    }),
                ..
            }, ..] => {}
            _ => {
                return Err(ErrorKind::TooLongMora);
            }
        }
    }
    Ok(moras)
}

fn generate_mora(consonant: Option<&Label>, vowel: &Label) -> crate::Mora {
    let consonant_phoneme = consonant.and_then(|c| c.phoneme.c.to_owned());
    let vowel = vowel.phoneme.c.clone().unwrap();
    crate::Mora {
        text: mora_to_text(consonant_phoneme.as_deref(), &vowel),
        consonant: consonant_phoneme,
        consonant_length: consonant.and(Some(0.0)),
        vowel,
        vowel_length: 0.0,
        pitch: 0.0,
    }
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
    mora2text(&mora_text).to_string()
}

fn mora2text(mora: &str) -> &str {
    MORA_PHONEMES_TO_MORA_KANA
        .get(mora)
        .map(Into::into)
        .unwrap_or(mora)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ::test_util::OPEN_JTALK_DIC_DIR;
    use jlabel::Label;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use rstest_reuse::*;

    use crate::AccentPhrase;

    use super::super::{
        full_context_label::{extract_full_context_label, generate_accent_phrases},
        open_jtalk::FullcontextExtractor,
        Mora,
    };

    fn mora(text: &str, consonant: Option<&str>, vowel: &str) -> Mora {
        Mora {
            text: text.into(),
            consonant: consonant.map(|c| c.into()),
            consonant_length: consonant.and(Some(0.0)),
            vowel: vowel.into(),
            vowel_length: 0.0,
            pitch: 0.0,
        }
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
            AccentPhrase {
                moras: vec![mora("イェ", Some("y"), "e")],
                accent: 1,
                pause_mora: None,
                is_interrogative: false,
            }
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
            AccentPhrase {
                moras: vec![
                    mora("ン", None, "N"),
                    mora("ン", None, "N"),
                    mora("ッ", None, "cl"),
                ],
                accent: 3,
                pause_mora: None,
                is_interrogative: false,
            },
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
            AccentPhrase {
                moras: vec![
                    mora("コ", Some("k"), "o"),
                    mora("レ", Some("r"), "e"),
                    mora("ワ", Some("w"), "a"),
                ],
                accent: 3,
                pause_mora: None,
                is_interrogative: false,
            },
            AccentPhrase {
                moras: vec![
                    mora("テ", Some("t"), "e"),
                    mora("ス", Some("s"), "U"),
                    mora("ト", Some("t"), "o"),
                    mora("デ", Some("d"), "e"),
                    mora("ス", Some("s"), "U"),
                ],
                accent: 1,
                pause_mora: None,
                is_interrogative: false,
            },
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
            AccentPhrase {
                moras: vec![
                    mora("イ", None, "i"),
                    mora("チ", Some("ch"), "i"),
                ],
                accent: 2,
                pause_mora: Some(mora("、", None, "pau")),
                is_interrogative: false,
            },
            AccentPhrase {
                moras: vec![
                    mora("セ", Some("s"), "e"),
                    mora("ン", None, "N"),
                ],
                accent: 1,
                pause_mora: Some(mora("、", None, "pau")),
                is_interrogative: false,
            },
            AccentPhrase {
                moras: vec![
                    mora("ヒャ", Some("hy"), "a"),
                    mora("ク", Some("k"), "u"),
                    mora("マ", Some("m"), "a"),
                    mora("ン", None, "N"),
                ],
                accent: 3,
                pause_mora: Some(mora("、", None, "pau")),
                is_interrogative: false,
            },
            AccentPhrase {
                moras: vec![
                    mora("イ", None, "i"),
                    mora("チ", Some("ch"), "i"),
                    mora("オ", None, "o"),
                    mora("ク", Some("k"), "u"),
                ],
                accent: 2,
                pause_mora: None,
                is_interrogative: true,
            },
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
            AccentPhrase {
                moras: vec![
                    mora("クヮ", Some("kw"), "a"),
                    mora("ル", Some("r"), "u"),
                    mora("テ", Some("t"), "e"),
                    mora("ッ", None, "cl"),
                    mora("ト", Some("t"), "o"),
                ],
                accent: 3,
                pause_mora: Some(mora("、", None, "pau")),
                is_interrogative: false,
            },
            AccentPhrase {
                moras: vec![
                    mora("ア", None, "a"),
                    mora("ア", None, "a"),
                ],
                accent: 1,
                pause_mora: None,
                is_interrogative: false,
            },
            AccentPhrase {
                moras: vec![mora("ア", None, "a")],
                accent: 1,
                pause_mora: None,
                is_interrogative: false,
            },
        ]
    )]
    fn label_cases(
        #[case] text: &str,
        #[case] labels: &[&str],
        #[case] accent_phrase: &[AccentPhrase],
    ) {
    }

    #[apply(label_cases)]
    #[tokio::test]
    async fn open_jtalk(text: &str, labels: &[&str], _accent_phrase: &[AccentPhrase]) {
        let open_jtalk = crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
            .await
            .unwrap();
        assert_eq!(&open_jtalk.0.extract_fullcontext(text).unwrap(), labels);
    }

    #[apply(label_cases)]
    fn parse_labels(_text: &str, labels: &[&str], accent_phrase: &[AccentPhrase]) {
        let parsed_labels = labels
            .iter()
            .map(|s| Label::from_str(s).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(
            &generate_accent_phrases(&parsed_labels).unwrap(),
            accent_phrase
        );
    }

    #[apply(label_cases)]
    #[tokio::test]
    async fn extract_fullcontext(text: &str, _labels: &[&str], accent_phrase: &[AccentPhrase]) {
        let open_jtalk = crate::nonblocking::OpenJtalk::new(OPEN_JTALK_DIC_DIR)
            .await
            .unwrap();
        assert_eq!(
            &extract_full_context_label(&open_jtalk.0, text).unwrap(),
            accent_phrase
        );
    }

    #[rstest]
    #[case("da", "ダ")]
    #[case("N", "ン")]
    #[case("cl", "ッ")]
    #[case("sho", "ショ")]
    #[case("u", "ウ")]
    #[case("fail", "fail")]
    fn test_mora2text(#[case] mora: &str, #[case] text: &str) {
        assert_eq!(super::mora2text(mora), text);
    }
}
