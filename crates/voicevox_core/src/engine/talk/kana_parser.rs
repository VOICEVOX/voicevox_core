use std::{collections::HashMap, sync::LazyLock};

use super::{
    super::{
        acoustic_feature_extractor::{Consonant, NonConsonant},
        mora_mappings::MORA_KANA_TO_MORA_PHONEMES,
    },
    AccentPhrase, Mora, ValidatedMora,
};

const UNVOICE_SYMBOL: char = '_';
const ACCENT_SYMBOL: char = '\'';
const NOPAUSE_DELIMITER: char = '/';
const PAUSE_DELIMITER: char = '、';
const WIDE_INTERROGATION_MARK: char = '？';
const LOOP_LIMIT: usize = 300;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("入力テキストをAquesTalk風記法としてパースすることに失敗しました: {_0}")]
pub(crate) struct KanaParseError(String);

type KanaParseResult<T> = std::result::Result<T, KanaParseError>;

static TEXT2MORA_WITH_UNVOICE: LazyLock<HashMap<String, ValidatedMora<'static>>> =
    LazyLock::new(|| {
        let mut text2mora_with_unvoice = HashMap::new();
        for (text, &(consonant, vowel)) in &MORA_KANA_TO_MORA_PHONEMES {
            let text = <&str>::from(text);
            let consonant = Option::<Consonant>::from(consonant).map(Into::into);

            if let Some(vowel) = vowel.to_unvoiced() {
                let unvoice_mora = ValidatedMora {
                    text: text.into(),
                    consonant: consonant.clone(),
                    vowel: NonConsonant::from(vowel).into(),
                    pitch: 0.,
                };
                text2mora_with_unvoice.insert(UNVOICE_SYMBOL.to_string() + text, unvoice_mora);
            }

            let mora = ValidatedMora {
                text: text.into(),
                consonant,
                vowel: NonConsonant::from(vowel).into(),
                pitch: 0.,
            };
            text2mora_with_unvoice.insert(text.to_string(), mora);
        }
        text2mora_with_unvoice
    });

fn text_to_accent_phrase(phrase: &str) -> KanaParseResult<AccentPhrase> {
    let phrase_vec: Vec<char> = phrase.chars().collect();
    let mut accent_index: Option<usize> = None;
    let mut moras: Vec<Mora> = Vec::new();
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
                    "accent cannot be set at beginning of accent phrase: {phrase}"
                )));
            }
            if accent_index.is_some() {
                return Err(KanaParseError(format!(
                    "second accent cannot be set at an accent phrase: {phrase}"
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
            moras.push(text2mora.get(&matched_text).unwrap().clone().into());
            stack.clear();
        } else {
            return Err(KanaParseError(format!(
                "unknown text in accent phrase: {phrase}"
            )));
        }
        if loop_count > LOOP_LIMIT {
            return Err(KanaParseError("detected infinity loop!".to_string()));
        }
    }
    if accent_index.is_none() {
        return Err(KanaParseError(format!(
            "accent not found in accent phrase: {phrase}"
        )));
    }
    Ok(AccentPhrase {
        moras,
        accent: accent_index.unwrap(),
        pause_mora: None,
        is_interrogative: false,
    })
}

pub(crate) fn parse_kana(text: &str) -> KanaParseResult<Vec<AccentPhrase>> {
    const TERMINATOR: char = '\0';
    if text.is_empty() {
        return Ok(vec![]);
    }
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
                        "interrogative mark cannot be set at not end of accent phrase: {phrase}"
                    )));
                }
                phrase.pop(); // remove WIDE_INTERROGATION_MARK
            }
            let accent_phrase = {
                let mut accent_phrase = text_to_accent_phrase(&phrase)?;
                if letter == PAUSE_DELIMITER {
                    accent_phrase.set_pause_mora(Some(Mora {
                        text: PAUSE_DELIMITER.to_string(),
                        consonant: None,
                        consonant_length: None,
                        vowel: "pau".to_string(),
                        vowel_length: 0.,
                        pitch: 0.,
                    }));
                }
                accent_phrase.set_is_interrogative(is_interrogative);
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

pub(crate) fn create_kana(accent_phrases: &[AccentPhrase]) -> String {
    let mut text = String::new();
    for phrase in accent_phrases {
        for (index, mora) in phrase.moras.iter().enumerate() {
            if ["A", "E", "I", "O", "U"].contains(&&*mora.vowel) {
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
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::super::super::mora_mappings::MORA_KANA_TO_MORA_PHONEMES;

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
        let text2mora = &super::TEXT2MORA_WITH_UNVOICE;
        assert_eq!(text2mora.len(), MORA_KANA_TO_MORA_PHONEMES.len() * 2 - 2); // added twice except ン and ッ
        let res = text2mora.get(text);
        assert_eq!(mora.is_some(), res.is_some());
        if let Some(res) = res {
            let mut m = String::new();
            if let Some(c) = &res.consonant {
                m.push_str(&c.phoneme.to_string());
            }
            m.push_str(&res.vowel.phoneme.to_string());
            assert_eq!(m, mora.unwrap());
        }
    }

    #[rstest]
    #[case("ア_シタ'ワ", true)]
    #[case("ユウヒガ'", true)]
    #[case("_キ'レイ", true)]
    #[case("アクセントナシ", false)]
    #[case("アクセ'ント'タクサン'", false)]
    #[case("'アクセントハジマリ", false)]
    #[case("不明な'文字", false)]
    fn test_text_to_accent_phrase(#[case] text: &str, #[case] result_is_ok_expected: bool) {
        let result = super::text_to_accent_phrase(text);
        assert_eq!(result.is_ok(), result_is_ok_expected, "{:?}", result);
    }

    #[rstest]
    #[case("テ'ス_ト/テ_ス'ト、_テ'_スト？/テ'ス_ト？", true)]
    #[case("クウハクノ'//フレーズ'", false)]
    #[case("フレー？ズノ'/トチュウニ'、ギモ'ンフ", false)]
    fn test_parse_kana(#[case] text: &str, #[case] result_is_ok_expected: bool) {
        let result = super::parse_kana(text);
        assert_eq!(result.is_ok(), result_is_ok_expected, "{:?}", result);
    }
    #[rstest]
    fn test_create_kana() {
        let text = "アンドロ'イドワ、デンキ'/ヒ'_ツジノ/ユメ'オ/ミ'ルカ？";
        let phrases = super::parse_kana(text).unwrap();
        let text_created = super::create_kana(&phrases);
        assert_eq!(text, &text_created);
    }
}
