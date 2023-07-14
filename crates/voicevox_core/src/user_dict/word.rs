use crate::result::Result;
use crate::user_dict::part_of_speech_data::{
    priority2cost, MAX_PRIORITY, MIN_PRIORITY, PART_OF_SPEECH_DETAIL,
};
use crate::Error;
use derive_getters::Getters;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// ユーザー辞書の単語。
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
pub struct UserDictWord {
    /// 単語の表記。
    pub surface: String,
    /// 単語の読み。
    pub pronunciation: String,
    /// アクセント型。
    pub accent_type: usize,
    /// 単語の種類。
    pub word_type: UserDictWordType,
    /// 単語の優先度。
    pub priority: u32,

    /// モーラ数。
    mora_count: usize,
}

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
        surface: String,
        pronunciation: String,
        accent_type: usize,
        word_type: UserDictWordType,
        priority: u32,
    ) -> Result<Self> {
        if MIN_PRIORITY > priority || priority > MAX_PRIORITY {
            return Err(Error::InvalidWord(format!(
                "優先度は{}以上{}以下である必要があります。",
                MIN_PRIORITY, MAX_PRIORITY
            )));
        }
        Self::validate_pronunciation(&pronunciation[..])?;
        let mora_count = Self::calculate_mora_count(&pronunciation[..], accent_type)?;
        Ok(Self {
            surface: Self::to_zenkaku(&surface[..]),
            pronunciation,
            accent_type,
            word_type,
            priority,
            mora_count,
        })
    }

    /// カタカナの文字列が発音として有効かどうかを判定する。
    fn validate_pronunciation(pronunciation: &str) -> Result<()> {
        // 元実装：https://github.com/VOICEVOX/voicevox_engine/blob/39747666aa0895699e188f3fd03a0f448c9cf746/voicevox_engine/model.py#L190-L210
        if !PRONUNCIATION_REGEX.is_match(pronunciation) {
            return Err(Error::InvalidWord(
                "発音は有効なカタカナである必要があります。".to_string(),
            ));
        }
        let sutegana = ['ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ャ', 'ュ', 'ョ', 'ヮ', 'ッ'];

        let pronunciation_chars = pronunciation.chars().collect::<Vec<_>>();

        for i in 0..pronunciation_chars.len() {
            // 「キャット」のように、捨て仮名が連続する可能性が考えられるので、
            // 「ッ」に関しては「ッ」そのものが連続している場合と、「ッ」の後にほかの捨て仮名が連続する場合のみ無効とする
            if sutegana.contains(pronunciation_chars.get(i).unwrap())
                && i < pronunciation_chars.len() - 1
                && (sutegana[..sutegana.len() - 1]
                    .contains(pronunciation_chars.get(i + 1).unwrap())
                    || (pronunciation_chars.get(i).unwrap() == &'ッ'
                        && sutegana.contains(pronunciation_chars.get(i + 1).unwrap())))
            {
                return Err(Error::InvalidWord(
                    "無効な発音です。(捨て仮名の連続)".to_string(),
                ));
            }

            if pronunciation_chars.get(i).unwrap() == &'ヮ'
                && i != 0
                && !['ク', 'グ'].contains(pronunciation_chars.get(i - 1).unwrap())
            {
                return Err(Error::InvalidWord(
                    "無効な発音です。(「くゎ」「ぐゎ」以外の「ゎ」の使用)".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// カタカナの発音からモーラ数を計算する。
    fn calculate_mora_count(pronunciation: &str, accent_type: usize) -> Result<usize> {
        // 元実装：https://github.com/VOICEVOX/voicevox_engine/blob/39747666aa0895699e188f3fd03a0f448c9cf746/voicevox_engine/model.py#L212-L236
        let mora_count = MORA_REGEX.find_iter(pronunciation).count();

        if accent_type > mora_count {
            return Err(Error::InvalidWord(format!(
                "誤ったアクセント型です({})。 expect: 0 <= accent_type <= {}",
                accent_type, mora_count
            )));
        }

        Ok(mora_count)
    }

    /// ASCII文字を全角文字に変換する。
    fn to_zenkaku(surface: &str) -> String {
        // 元実装：https://github.com/VOICEVOX/voicevox/blob/69898f5dd001d28d4de355a25766acb0e0833ec2/src/components/DictionaryManageDialog.vue#L379-L387
        let mut result = String::new();
        for c in surface.chars() {
            let i = c as u32;
            result.push(if (0x21..=0x7e).contains(&i) {
                char::from_u32(0xfee0 + i).unwrap_or(c)
            } else if i == 0x20 {
                '　'
            } else {
                c
            });
        }
        result
    }
}

/// ユーザー辞書の単語の種類。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
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
    pub fn to_mecab_format(&self) -> String {
        let pos = PART_OF_SPEECH_DETAIL.get(&self.word_type).unwrap();
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}/{},{}\n",
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
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("abcdefg", "ａｂｃｄｅｆｇ")]
    #[case("あいうえお", "あいうえお")]
    #[case("a_b_c_d_e_f_g", "ａ＿ｂ＿ｃ＿ｄ＿ｅ＿ｆ＿ｇ")]
    #[case("a b c d e f g", "ａ　ｂ　ｃ　ｄ　ｅ　ｆ　ｇ")]
    fn to_zenkaku_works(#[case] before: &str, #[case] after: &str) {
        assert_eq!(UserDictWord::to_zenkaku(before), after);
    }

    #[rstest]
    fn to_mecab_format_works() {
        // テストの期待値は、VOICEVOX Engineが一時的に出力するcsvの内容を使用した。
        let word = UserDictWord::new(
            "単語".to_string(),
            "ヨミ".to_string(),
            0,
            UserDictWordType::ProperNoun,
            5,
        )
        .unwrap();
        assert_eq!(
            word.to_mecab_format(),
            "単語,1348,1348,8609,名詞,固有名詞,一般,*,*,*,*,ヨミ,ヨミ,0/2,*\n"
        );
    }
}
