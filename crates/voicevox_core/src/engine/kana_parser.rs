const UNVOICE_SYMBOL: char = '_';
const ACCENT_SYMBOL: char = '\'';
const NOPAUSE_DELIMITER: char = '/';
const PAUSE_DELIMITER: char = '、';
const WIDE_INTERROGATION_MARK: char = '？';
const LOOP_LIMIT: usize = 300;

#[allow(dead_code)] // TODO: remove this feature
#[derive(Clone)]
struct MoraModel {
    text: String,
    consonant: Option<String>,
    consonant_length: Option<f32>,
    vowel: String,
    vowel_length: f32,
    pitch: f32,
}

#[allow(dead_code)] // TODO: remove this feature
struct AccentPhraseModel {
    moras: Vec<MoraModel>,
    accent: usize,
    pause_mora: Option<MoraModel>,
    is_interrogative: bool,
}

#[allow(dead_code)] // TODO: remove this feature
struct AudioQueryModel {
    accent_phrases: Vec<AccentPhraseModel>,
    speed_scale: f32,
    pitch_scale: f32,
    intonation_scale: f32,
    volume_scale: f32,
    pre_phoneme_length: f32,
    post_phoneme_length: f32,
    output_sampling_rate: u32,
    output_stereo: bool,
    kana: String,
}

#[derive(Clone, Debug)]
struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse Error: {}", self.0)
    }
}

impl std::error::Error for ParseError {}

const MORA_LIST_MINIMUM: [[&str; 3]; 144] = [
    ["ヴォ", "v", "o"],
    ["ヴェ", "v", "e"],
    ["ヴィ", "v", "i"],
    ["ヴァ", "v", "a"],
    ["ヴ", "v", "u"],
    ["ン", "", "N"],
    ["ワ", "w", "a"],
    ["ロ", "r", "o"],
    ["レ", "r", "e"],
    ["ル", "r", "u"],
    ["リョ", "ry", "o"],
    ["リュ", "ry", "u"],
    ["リャ", "ry", "a"],
    ["リェ", "ry", "e"],
    ["リ", "r", "i"],
    ["ラ", "r", "a"],
    ["ヨ", "y", "o"],
    ["ユ", "y", "u"],
    ["ヤ", "y", "a"],
    ["モ", "m", "o"],
    ["メ", "m", "e"],
    ["ム", "m", "u"],
    ["ミョ", "my", "o"],
    ["ミュ", "my", "u"],
    ["ミャ", "my", "a"],
    ["ミェ", "my", "e"],
    ["ミ", "m", "i"],
    ["マ", "m", "a"],
    ["ポ", "p", "o"],
    ["ボ", "b", "o"],
    ["ホ", "h", "o"],
    ["ペ", "p", "e"],
    ["ベ", "b", "e"],
    ["ヘ", "h", "e"],
    ["プ", "p", "u"],
    ["ブ", "b", "u"],
    ["フォ", "f", "o"],
    ["フェ", "f", "e"],
    ["フィ", "f", "i"],
    ["ファ", "f", "a"],
    ["フ", "f", "u"],
    ["ピョ", "py", "o"],
    ["ピュ", "py", "u"],
    ["ピャ", "py", "a"],
    ["ピェ", "py", "e"],
    ["ピ", "p", "i"],
    ["ビョ", "by", "o"],
    ["ビュ", "by", "u"],
    ["ビャ", "by", "a"],
    ["ビェ", "by", "e"],
    ["ビ", "b", "i"],
    ["ヒョ", "hy", "o"],
    ["ヒュ", "hy", "u"],
    ["ヒャ", "hy", "a"],
    ["ヒェ", "hy", "e"],
    ["ヒ", "h", "i"],
    ["パ", "p", "a"],
    ["バ", "b", "a"],
    ["ハ", "h", "a"],
    ["ノ", "n", "o"],
    ["ネ", "n", "e"],
    ["ヌ", "n", "u"],
    ["ニョ", "ny", "o"],
    ["ニュ", "ny", "u"],
    ["ニャ", "ny", "a"],
    ["ニェ", "ny", "e"],
    ["ニ", "n", "i"],
    ["ナ", "n", "a"],
    ["ドゥ", "d", "u"],
    ["ド", "d", "o"],
    ["トゥ", "t", "u"],
    ["ト", "t", "o"],
    ["デョ", "dy", "o"],
    ["デュ", "dy", "u"],
    ["デャ", "dy", "a"],
    ["ディ", "d", "i"],
    ["デ", "d", "e"],
    ["テョ", "ty", "o"],
    ["テュ", "ty", "u"],
    ["テャ", "ty", "a"],
    ["ティ", "t", "i"],
    ["テ", "t", "e"],
    ["ツォ", "ts", "o"],
    ["ツェ", "ts", "e"],
    ["ツィ", "ts", "i"],
    ["ツァ", "ts", "a"],
    ["ツ", "ts", "u"],
    ["ッ", "", "cl"],
    ["チョ", "ch", "o"],
    ["チュ", "ch", "u"],
    ["チャ", "ch", "a"],
    ["チェ", "ch", "e"],
    ["チ", "ch", "i"],
    ["ダ", "d", "a"],
    ["タ", "t", "a"],
    ["ゾ", "z", "o"],
    ["ソ", "s", "o"],
    ["ゼ", "z", "e"],
    ["セ", "s", "e"],
    ["ズィ", "z", "i"],
    ["ズ", "z", "u"],
    ["スィ", "s", "i"],
    ["ス", "s", "u"],
    ["ジョ", "j", "o"],
    ["ジュ", "j", "u"],
    ["ジャ", "j", "a"],
    ["ジェ", "j", "e"],
    ["ジ", "j", "i"],
    ["ショ", "sh", "o"],
    ["シュ", "sh", "u"],
    ["シャ", "sh", "a"],
    ["シェ", "sh", "e"],
    ["シ", "sh", "i"],
    ["ザ", "z", "a"],
    ["サ", "s", "a"],
    ["ゴ", "g", "o"],
    ["コ", "k", "o"],
    ["ゲ", "g", "e"],
    ["ケ", "k", "e"],
    ["グヮ", "gw", "a"],
    ["グ", "g", "u"],
    ["クヮ", "kw", "a"],
    ["ク", "k", "u"],
    ["ギョ", "gy", "o"],
    ["ギュ", "gy", "u"],
    ["ギャ", "gy", "a"],
    ["ギェ", "gy", "e"],
    ["ギ", "g", "i"],
    ["キョ", "ky", "o"],
    ["キュ", "ky", "u"],
    ["キャ", "ky", "a"],
    ["キェ", "ky", "e"],
    ["キ", "k", "i"],
    ["ガ", "g", "a"],
    ["カ", "k", "a"],
    ["オ", "", "o"],
    ["エ", "", "e"],
    ["ウォ", "w", "o"],
    ["ウェ", "w", "e"],
    ["ウィ", "w", "i"],
    ["ウ", "", "u"],
    ["イェ", "y", "e"],
    ["イ", "", "i"],
    ["ア", "", "a"],
];

#[allow(dead_code)] // TODO: remove this feature
fn mora2text(mora: &str) -> &str {
    for [text, consonant, vowel] in MORA_LIST_MINIMUM {
        if mora.len() >= consonant.len()
            && &mora[..consonant.len()] == consonant
            && &mora[consonant.len()..] == vowel
        {
            return text;
        }
    }
    mora
}

fn text2mora_with_unvioce() -> std::collections::BTreeMap<String, MoraModel> {
    let mut text2mora_with_unvioce = std::collections::BTreeMap::new();
    for [text, consonant, vowel] in MORA_LIST_MINIMUM {
        let consonant = if consonant.is_empty() {
            Some(consonant.to_string())
        } else {
            None
        };
        let consonant_length = if consonant.is_some() { Some(0.0) } else { None };

        if ["a", "i", "u", "e", "o"].contains(&vowel) {
            let upper_vowel = vowel.chars().next().unwrap().to_uppercase().to_string();
            let unvoice_mora = MoraModel {
                text: text.to_string(),
                consonant: consonant.clone(),
                consonant_length,
                vowel: upper_vowel,
                vowel_length: 0.0,
                pitch: 0.0,
            };
            text2mora_with_unvioce.insert(UNVOICE_SYMBOL.to_string() + text, unvoice_mora);
        }

        let mora = MoraModel {
            text: text.to_string(),
            consonant,
            consonant_length,
            vowel: vowel.to_string(),
            vowel_length: 0.0,
            pitch: 0.0,
        };
        text2mora_with_unvioce.insert(text.to_string(), mora);
    }
    text2mora_with_unvioce
}

fn text_to_accent_phrase(phrase: &str) -> Result<AccentPhraseModel, ParseError> {
    let phrase_vec: Vec<char> = phrase.chars().collect();
    let mut accent_index: Option<usize> = None;
    let mut moras: Vec<MoraModel> = Vec::new();
    let mut stack = String::new();
    let mut matched_text: Option<String> = None;
    let text2mora = text2mora_with_unvioce();
    let mut index = 0;
    let mut loop_count = 0;
    while index < phrase_vec.len() {
        loop_count += 1;
        let letter = phrase_vec[index];
        if letter == ACCENT_SYMBOL {
            if index == 0 {
                return Err(ParseError(format!(
                    "accent cannot be set at beginning of accent phrase: {}",
                    phrase
                )));
            }
            if accent_index.is_some() {
                return Err(ParseError(format!(
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
            return Err(ParseError(format!(
                "unknown text in accent phrase: {}",
                phrase
            )));
        }
        if loop_count > LOOP_LIMIT {
            return Err(ParseError("detected infinity loop!".to_string()));
        }
    }
    if accent_index.is_none() {
        return Err(ParseError(format!(
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
fn parse_kana(text: &str) -> Result<Vec<AccentPhraseModel>, ParseError> {
    const DUMMY: char = '\0';
    let mut parsed_result = Vec::new();
    let text_vec: Vec<char> = text.chars().chain([DUMMY]).collect();
    let mut phrase = String::new();
    for letter in text_vec {
        if letter == DUMMY || letter == PAUSE_DELIMITER || letter == NOPAUSE_DELIMITER {
            if phrase.is_empty() {
                return Err(ParseError(format!(
                    "accent phrase at position of {} is empty",
                    parsed_result.len()
                )));
            }
            let is_interrogative = phrase.contains(WIDE_INTERROGATION_MARK);
            if is_interrogative {
                if phrase.find(WIDE_INTERROGATION_MARK).unwrap() == phrase.len() - 1 {
                    return Err(ParseError(format!(
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
    #[test]
    fn test_mora2text() {
        let lis = [
            ("da", "ダ"),
            ("N", "ン"),
            ("cl", "ッ"),
            ("sho", "ショ"),
            ("u", "ウ"),
            ("fail", "fail"),
        ];
        for (mora, text) in lis {
            assert_eq!(mora2text(mora), text);
        }
    }

    #[test]
    fn test_text2mora_with_unvoice() {
        let text2mora = text2mora_with_unvioce();
        assert_eq!(text2mora.len(), MORA_LIST_MINIMUM.len() * 2 - 2); // added twice except ン and ッ
        let lis = [
            (Some("da"), "ダ"),
            (Some("N"), "ン"),
            (Some("cl"), "ッ"),
            (Some("sho"), "ショ"),
            (Some("u"), "ウ"),
            (Some("gA"), "_ガ"),
            (Some("byO"), "_ビョ"),
            (Some("O"), "_オ"),
            (None, "fail"),
        ];
        for (mora, text) in lis {
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
            assert!(text_to_accent_phrase(text).is_ok());
            // TODO: もっと細かい確認
        }
        for text in text_err {
            assert!(text_to_accent_phrase(text).is_err());
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
            assert!(parse_kana(text).is_ok());
            // TODO: もっと細かい確認
        }
        for text in text_err {
            assert!(parse_kana(text).is_err());
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
