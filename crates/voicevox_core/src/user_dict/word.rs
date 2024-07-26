use crate::{
    error::ErrorRepr,
    result::Result,
    user_dict::part_of_speech_data::{
        priority2cost, MAX_PRIORITY, MIN_PRIORITY, PART_OF_SPEECH_DETAIL,
    },
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{de::Error as _, Deserialize, Serialize};
use std::ops::RangeToInclusive;

/// ユーザー辞書の単語。
#[derive(Clone, Debug, Serialize)]
pub struct UserDictWord {
    /// 単語の表記。
    surface: String,
    /// 単語の読み。
    pronunciation: String,
    /// アクセント型。
    accent_type: usize,
    /// 単語の種類。
    word_type: UserDictWordType,
    /// 単語の優先度。
    priority: u32,

    /// モーラ数。
    mora_count: usize,
}

impl<'de> Deserialize<'de> for UserDictWord {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = UserDictWord::deserialize(deserializer)?;
        return Self::new(
            &raw.surface,
            raw.pronunciation,
            raw.accent_type,
            raw.word_type,
            raw.priority,
        )
        .map_err(D::Error::custom);

        #[derive(Deserialize)]
        struct UserDictWord {
            surface: String,
            pronunciation: String,
            accent_type: usize,
            word_type: UserDictWordType,
            priority: u32,
        }
    }
}

#[allow(clippy::enum_variant_names)] // FIXME
#[derive(thiserror::Error, Debug, PartialEq)]
pub(crate) enum InvalidWordError {
    #[error("{}: 無効な発音です({_1}): {_0:?}", Self::BASE_MSG)]
    InvalidPronunciation(String, &'static str),
    #[error(
        "{}: 優先度は{MIN_PRIORITY}以上{MAX_PRIORITY}以下である必要があります: {_0}",
        Self::BASE_MSG
    )]
    InvalidPriority(u32),
    #[error(
        "{}: 誤ったアクセント型です({1:?}の範囲から外れています): {_0}",
        Self::BASE_MSG
    )]
    InvalidAccentType(usize, RangeToInclusive<usize>),
}

impl InvalidWordError {
    const BASE_MSG: &'static str = "ユーザー辞書の単語のバリデーションに失敗しました";
}

type InvalidWordResult<T> = std::result::Result<T, InvalidWordError>;

static PRONUNCIATION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[ァ-ヴー]+$").unwrap());
static MORA_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        "(?:",
        "[イ][ェ]|[ヴ][ャュョ]|[トド][ゥ]|[テデ][ィャュョ]|[デ][ェ]|[クグ][ヮ]|", // rule_others
        "[キシチニヒミリギジビピ][ェャュョ]|",                                    // rule_line_i
        "[ツフヴ][ァ]|[ウスツフヴズ][ィ]|[ウツフヴ][ェォ]|",                      // rule_line_u
        "[ァ-ヴー]",                                                              // rule_one_mora
        ")",
    ))
    .unwrap()
});
static SPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\p{Z}").unwrap());

impl Default for UserDictWord {
    fn default() -> Self {
        Self {
            surface: "".to_string(),
            pronunciation: "".to_string(),
            accent_type: 0,
            word_type: UserDictWordType::CommonNoun,
            priority: 0,
            mora_count: 0,
        }
    }
}

impl UserDictWord {
    pub fn new(
        surface: &str,
        pronunciation: String,
        accent_type: usize,
        word_type: UserDictWordType,
        priority: u32,
    ) -> Result<Self> {
        if MIN_PRIORITY > priority || priority > MAX_PRIORITY {
            return Err(ErrorRepr::InvalidWord(InvalidWordError::InvalidPriority(priority)).into());
        }
        validate_pronunciation(&pronunciation)?;
        let mora_count = calculate_mora_count(&pronunciation, accent_type)?;
        Ok(Self {
            surface: to_zenkaku(surface),
            pronunciation,
            accent_type,
            word_type,
            priority,
            mora_count,
        })
    }

    /// 単語の表記。
    pub fn surface(&self) -> &str {
        &self.surface
    }

    /// 単語の読み。
    pub fn pronunciation(&self) -> &str {
        &self.pronunciation
    }

    /// アクセント型。
    pub fn accent_type(&self) -> usize {
        self.accent_type
    }

    /// 単語の種類。
    pub fn word_type(&self) -> UserDictWordType {
        self.word_type
    }

    /// 単語の優先度。
    pub fn priority(&self) -> u32 {
        self.priority
    }
}

/// カタカナの文字列が発音として有効かどうかを判定する。
pub(crate) fn validate_pronunciation(pronunciation: &str) -> InvalidWordResult<()> {
    // 元実装：https://github.com/VOICEVOX/voicevox_engine/blob/39747666aa0895699e188f3fd03a0f448c9cf746/voicevox_engine/model.py#L190-L210
    if !PRONUNCIATION_REGEX.is_match(pronunciation) {
        return Err(InvalidWordError::InvalidPronunciation(
            pronunciation.to_string(),
            "カタカナ以外の文字",
        ));
    }
    let sutegana = ['ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ャ', 'ュ', 'ョ', 'ヮ', 'ッ'];

    let pronunciation_chars = pronunciation.chars().collect::<Vec<_>>();

    for i in 0..pronunciation_chars.len() {
        // 「キャット」のように、捨て仮名が連続する可能性が考えられるので、
        // 「ッ」に関しては「ッ」そのものが連続している場合と、「ッ」の後にほかの捨て仮名が連続する場合のみ無効とする
        if sutegana.contains(&pronunciation_chars[i])
            && i < pronunciation_chars.len() - 1
            && (sutegana[..sutegana.len() - 1].contains(pronunciation_chars.get(i + 1).unwrap())
                || (pronunciation_chars.get(i).unwrap() == &'ッ'
                    && sutegana.contains(pronunciation_chars.get(i + 1).unwrap())))
        {
            return Err(InvalidWordError::InvalidPronunciation(
                pronunciation.to_string(),
                "捨て仮名の連続",
            ));
        }

        if pronunciation_chars.get(i).unwrap() == &'ヮ'
            && i != 0
            && !['ク', 'グ'].contains(&pronunciation_chars[i - 1])
        {
            return Err(InvalidWordError::InvalidPronunciation(
                pronunciation.to_string(),
                "「くゎ」「ぐゎ」以外の「ゎ」の使用",
            ));
        }
    }
    Ok(())
}

/// カタカナの発音からモーラ数を計算する。
fn calculate_mora_count(pronunciation: &str, accent_type: usize) -> InvalidWordResult<usize> {
    // 元実装：https://github.com/VOICEVOX/voicevox_engine/blob/39747666aa0895699e188f3fd03a0f448c9cf746/voicevox_engine/model.py#L212-L236
    let mora_count = MORA_REGEX.find_iter(pronunciation).count();

    if accent_type > mora_count {
        return Err(InvalidWordError::InvalidAccentType(
            accent_type,
            ..=mora_count,
        ));
    }

    Ok(mora_count)
}

/// 一部の種類の文字を、全角文字に置き換える。
///
/// 具体的には
///
/// - "!"から"~"までの範囲の文字(数字やアルファベット)は、対応する全角文字に
/// - " "などの目に見えない文字は、まとめて全角スペース(0x3000)に
///
/// 変換する。
pub(crate) fn to_zenkaku(surface: &str) -> String {
    // 元実装：https://github.com/VOICEVOX/voicevox/blob/69898f5dd001d28d4de355a25766acb0e0833ec2/src/components/DictionaryManageDialog.vue#L379-L387
    SPACE_REGEX
        .replace_all(surface, "\u{3000}")
        .chars()
        .map(|c| match u32::from(c) {
            i @ 0x21..=0x7e => char::from_u32(0xfee0 + i).unwrap_or(c),
            _ => c,
        })
        .collect()
}
/// ユーザー辞書の単語の種類。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserDictWordType {
    /// 固有名詞。
    ProperNoun,
    /// 一般名詞。
    CommonNoun,
    /// 動詞。
    Verb,
    /// 形容詞。
    Adjective,
    /// 接尾辞。
    Suffix,
}

impl UserDictWord {
    pub(super) fn to_mecab_format(&self) -> String {
        let pos = PART_OF_SPEECH_DETAIL.get(&self.word_type).unwrap();
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}/{},{}",
            self.surface,
            pos.context_id,
            pos.context_id,
            priority2cost(pos.context_id, self.priority),
            pos.part_of_speech,
            pos.part_of_speech_detail_1,
            pos.part_of_speech_detail_2,
            pos.part_of_speech_detail_3,
            "*",                // inflectional_type
            "*",                // inflectional_form
            "*",                // stem
            self.pronunciation, // yomi
            self.pronunciation,
            self.accent_type,
            self.mora_count,
            "*" // accent_associative_rule
        )
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{InvalidWordError, UserDictWord, UserDictWordType};

    #[rstest]
    #[case("abcdefg", "ａｂｃｄｅｆｇ")]
    #[case("あいうえお", "あいうえお")]
    #[case("a_b_c_d_e_f_g", "ａ＿ｂ＿ｃ＿ｄ＿ｅ＿ｆ＿ｇ")]
    #[case("a b c d e f g", "ａ　ｂ　ｃ　ｄ　ｅ　ｆ　ｇ")]
    fn to_zenkaku_works(#[case] before: &str, #[case] after: &str) {
        assert_eq!(super::to_zenkaku(before), after);
    }

    #[rstest]
    fn to_mecab_format_works() {
        // テストの期待値は、VOICEVOX Engineが一時的に出力するcsvの内容を使用した。
        let word = UserDictWord::new(
            "単語",
            "ヨミ".to_string(),
            0,
            UserDictWordType::ProperNoun,
            5,
        )
        .unwrap();
        assert_eq!(
            word.to_mecab_format(),
            "単語,1348,1348,8609,名詞,固有名詞,一般,*,*,*,*,ヨミ,ヨミ,0/2,*"
        );
    }

    #[rstest]
    #[case("ヨミ", None)]
    #[case("漢字", Some("カタカナ以外の文字"))]
    #[case("ひらがな", Some("カタカナ以外の文字"))]
    #[case("ッッッ", Some("捨て仮名の連続"))]
    #[case("ァァァァ", Some("捨て仮名の連続"))]
    #[case("ヌヮ", Some("「くゎ」「ぐゎ」以外の「ゎ」の使用"))]
    fn pronunciation_validation_works(
        #[case] pronunciation: &str,
        #[case] expected_error_message: Option<&str>,
    ) {
        let result = super::validate_pronunciation(pronunciation);

        if let Some(expected_error_message) = expected_error_message {
            match result {
                Ok(_) => unreachable!(),
                Err(InvalidWordError::InvalidPronunciation(err_pronunciation, err_message)) => {
                    assert_eq!(err_pronunciation, pronunciation);
                    assert_eq!(err_message, expected_error_message);
                }
                Err(_) => unreachable!(),
            }
        } else {
            assert!(result.is_ok());
        }
    }
}
