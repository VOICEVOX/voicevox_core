use crate::engine::model::{AccentPhraseModel, MoraModel};
use crate::engine::mora_list::MORA_LIST_MINIMUM;
use once_cell::sync::Lazy;
use std::collections::HashMap;

const UNVOICE_SYMBOL: char = '_';
const ACCENT_SYMBOL: char = '\'';
const NOPAUSE_DELIMITER: char = '/';
const PAUSE_DELIMITER: char = '、';
const WIDE_INTERROGATION_MARK: char = '？';
const LOOP_LIMIT: usize = 300;

#[derive(Clone, Debug)]
struct KanaParseError(String);

impl std::fmt::Display for KanaParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse Error: {}", self.0)
    }
}

impl std::error::Error for KanaParseError {}

type KanaParseResult<T> = std::result::Result<T, KanaParseError>;

static TEXT2MORA_WITH_UNVOICE: Lazy<HashMap<String, MoraModel>> = Lazy::new(|| {
    let mut text2mora_with_unvoice = HashMap::new();
    for [text, consonant, vowel] in MORA_LIST_MINIMUM {
        let consonant = if !consonant.is_empty() {
            Some(consonant.to_string())
        } else {
            None
        };
        let consonant_length = if consonant.is_some() { Some(0.0) } else { None };

        if ["a", "i", "u", "e", "o"].contains(vowel) {
            let upper_vowel = vowel.to_uppercase();
            let unvoice_mora = MoraModel {
                text: text.to_string(),
                consonant: consonant.clone(),
                consonant_length,
                vowel: upper_vowel,
                vowel_length: 0.0,
                pitch: 0.0,
            };
            text2mora_with_unvoice.insert(UNVOICE_SYMBOL.to_string() + text, unvoice_mora);
        }

        let mora = MoraModel {
            text: text.to_string(),
            consonant,
            consonant_length,
            vowel: vowel.to_string(),
            vowel_length: 0.0,
            pitch: 0.0,
        };
        text2mora_with_unvoice.insert(text.to_string(), mora);
    }
    text2mora_with_unvoice
});

fn text_to_accent_phrase(phrase: &str) -> KanaParseResult<AccentPhraseModel> {
    let phrase_vec: Vec<char> = phrase.chars().collect();
    let mut accent_index: Option<usize> = None;
    let mut moras: Vec<MoraModel> = Vec::new();
    let mut stack = String::new();
    let mut matched_text: Option<String> = None;
    let text2mora = &TEXT2MORA_WITH_UNVOICE;
    let mut index = 0;
    let mut loop_count = 0;
    while index < phrase_vec.len() {
        loop_count += 1;
        let letter = phrase_vec[index];
        if letter == ACCENT_SYMBOL {
            if index == 0 {
                return Err(KanaParseError(format!(
                    "accent cannot be set at beginning of accent phrase: {}",
                    phrase
                )));
            }
            if accent_index.is_some() {
                return Err(KanaParseError(format!(
                    "second accent cannot be set at an accent phrase: {}",
                    phrase
                )));
            }
            accent_index = Some(moras.len());
            index += 1;
            continue;
        }

        for &watch_letter in &phrase_vec[index..] {
            if watch_letter == ACCENT_SYMBOL {
                break;
            }
            stack.push(watch_letter);
            if text2mora.contains_key(&stack) {
                matched_text = Some(stack.clone());
            }
        }
        if let Some(matched_text) = matched_text.take() {
            index += matched_text.chars().count();
            moras.push(text2mora.get(&matched_text).unwrap().clone());
            stack.clear();
        } else {
            return Err(KanaParseError(format!(
                "unknown text in accent phrase: {}",
                phrase
            )));
        }
        if loop_count > LOOP_LIMIT {
            return Err(KanaParseError("detected infinity loop!".to_string()));
        }
    }
    if accent_index.is_none() {
        return Err(KanaParseError(format!(
            "accent not found in accent phrase: {}",
            phrase
        )));
    }
    Ok(AccentPhraseModel {
        moras,
        accent: accent_index.unwrap(),
        pause_mora: None,
        is_interrogative: false,
    })
}

#[allow(dead_code)] // TODO: remove this feature
fn parse_kana(text: &str) -> KanaParseResult<Vec<AccentPhraseModel>> {
    const TERMINATOR: char = '\0';
    let mut parsed_result = Vec::new();
    let chars_of_text = text.chars().chain([TERMINATOR]);
    let mut phrase = String::new();
    for letter in chars_of_text {
        if letter == TERMINATOR || letter == PAUSE_DELIMITER || letter == NOPAUSE_DELIMITER {
            if phrase.is_empty() {
                return Err(KanaParseError(format!(
                    "accent phrase at position of {} is empty",
                    parsed_result.len()
                )));
            }
            let is_interrogative = phrase.contains(WIDE_INTERROGATION_MARK);
            if is_interrogative {
                if phrase.find(WIDE_INTERROGATION_MARK).unwrap()
                    != phrase.len() - WIDE_INTERROGATION_MARK.len_utf8()
                {
                    return Err(KanaParseError(format!(
                        "interrogative mark cannot be set at not end of accent phrase: {}",
                        phrase
                    )));
                }
                phrase.pop(); // remove WIDE_INTERROGATION_MARK
            }
            let accent_phrase = {
                let mut accent_phrase = text_to_accent_phrase(&phrase)?;
                if letter == PAUSE_DELIMITER {
                    accent_phrase.pause_mora = Some(MoraModel {
                        text: PAUSE_DELIMITER.to_string(),
                        consonant: None,
                        consonant_length: None,
                        vowel: "pau".to_string(),
                        vowel_length: 0.0,
                        pitch: 0.0,
                    });
                }
                accent_phrase.is_interrogative = is_interrogative;
                accent_phrase
            };
            parsed_result.push(accent_phrase);
            phrase.clear();
        } else {
            phrase.push(letter);
        }
    }
    Ok(parsed_result)
}

#[allow(dead_code)] // TODO: remove this feature
fn create_kana(accent_phrases: &[AccentPhraseModel]) -> String {
    let mut text = String::new();
    for phrase in accent_phrases {
        let moras = &phrase.moras;
        for (index, mora) in moras.iter().enumerate() {
            if ["A", "E", "I", "O", "U"].contains(&mora.vowel.as_ref()) {
                text.push(UNVOICE_SYMBOL);
            }
            text.push_str(&mora.text);
            if index + 1 == phrase.accent {
                text.push(ACCENT_SYMBOL);
            }
        }
        if phrase.is_interrogative {
            text.push(WIDE_INTERROGATION_MARK);
        }
        text.push(if phrase.pause_mora.is_some() {
            PAUSE_DELIMITER
        } else {
            NOPAUSE_DELIMITER
        });
    }
    text.pop(); // remove last delimiter
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::mora_list::MORA_LIST_MINIMUM;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case(Some("da"), "ダ")]
    #[case(Some("N"), "ン")]
    #[case(Some("cl"), "ッ")]
    #[case(Some("sho"), "ショ")]
    #[case(Some("u"), "ウ")]
    #[case(Some("gA"), "_ガ")]
    #[case(Some("byO"), "_ビョ")]
    #[case(Some("O"), "_オ")]
    #[case(None, "fail")]
    fn test_text2mora_with_unvoice(#[case] mora: Option<&str>, #[case] text: &str) {
        let text2mora = &TEXT2MORA_WITH_UNVOICE;
        assert_eq!(text2mora.len(), MORA_LIST_MINIMUM.len() * 2 - 2); // added twice except ン and ッ
        let res = text2mora.get(text);
        assert_eq!(mora.is_some(), res.is_some());
        if let Some(res) = res {
            let mut m = String::new();
            if let Some(ref c) = res.consonant {
                m.push_str(c);
            }
            m.push_str(&res.vowel);
            assert_eq!(m, mora.unwrap());
        }
    }

    #[test]
    fn test_text_to_accent_phrase() {
        let text_ok = ["ア_シタ'ワ", "ユウヒガ'", "_キ'レイ"];
        let text_err = [
            "アクセントナシ",
            "アクセ'ント'タクサン'",
            "'アクセントハジマリ",
            "不明な'文字",
        ];
        for text in text_ok {
            let result = text_to_accent_phrase(text);
            assert!(result.is_ok(), "{:?}", result);
        }
        for text in text_err {
            let result = text_to_accent_phrase(text);
            assert!(result.is_err(), "{:?}", result);
        }
    }

    #[test]
    fn test_parse_kana() {
        let text_ok = ["テ'ス_ト/テ_ス'ト、_テ'_スト？/テ'ス_ト？"];
        let text_err = [
            "クウハクノ'//フレーズ'",
            "フレー？ズノ'/トチュウニ'、ギモ'ンフ",
        ];
        for text in text_ok {
            let result = parse_kana(text);
            assert!(result.is_ok(), "{:?}", result);
        }
        for text in text_err {
            let result = parse_kana(text);
            assert!(result.is_err(), "{:?}", result);
        }
    }
    #[test]
    fn test_create_kana() {
        let text = "アンドロ'イドワ、デンキ'/ヒ'_ツジノ/ユメ'オ/ミ'ルカ？";
        let phrases = parse_kana(text).unwrap();
        let text_created = create_kana(&phrases);
        assert_eq!(text, &text_created);
    }
}
