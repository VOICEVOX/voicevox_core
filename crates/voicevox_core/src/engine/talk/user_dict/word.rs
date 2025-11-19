use std::{ops::RangeToInclusive, sync::LazyLock};

use regex::Regex;
use serde::{de::Error as _, Deserialize, Serialize, Serializer};

use crate::{error::ErrorRepr, result::Result};

use super::{
    super::text::{hankaku_zenkaku, katakana},
    part_of_speech_data::{
        priority2cost, PartOfSpeechDetail, MAX_PRIORITY, MIN_PRIORITY, PART_OF_SPEECH_DETAIL,
    },
};

/// ユーザー辞書の単語。
///
/// # Serialization
///
/// VOICEVOX ENGINEと同じスキーマになっている。ただし今後の破壊的変更にて変わる可能性がある。[データのシリアライゼーション]を参照。
///
/// [データのシリアライゼーション]: https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md
#[cfg_attr(doc, doc(alias = "VoicevoxUserDictWord"))]
#[derive(Clone, PartialEq, Debug)]
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
        let SerdeRepr {
            surface,
            priority,
            context_id,
            part_of_speech,
            part_of_speech_detail_1,
            part_of_speech_detail_2,
            part_of_speech_detail_3,
            inflectional_type,
            inflectional_form,
            stem,
            yomi,
            pronunciation,
            accent_type,
            mora_count,
            accent_associative_rule,
        } = SerdeRepr::<String>::deserialize(deserializer)?;

        if inflectional_type != "*" {
            return Err(D::Error::custom("`inflectional_type` must be \"*\""));
        }
        if inflectional_form != "*" {
            return Err(D::Error::custom("`inflectional_form` must be \"*\""));
        }
        if stem != "*" {
            return Err(D::Error::custom("`stem` must be \"*\""));
        }
        if yomi != pronunciation {
            return Err(D::Error::custom("`yomi` must equal to `pronunciation`"));
        }
        if accent_associative_rule != "*" {
            return Err(D::Error::custom("`accent_associative_rule` must be \"*\""));
        }

        let (word_type, _) = PART_OF_SPEECH_DETAIL
            .iter()
            .find(|(_, pos)| {
                part_of_speech == pos.part_of_speech
                    && part_of_speech_detail_1 == pos.part_of_speech_detail_1
                    && part_of_speech_detail_2 == pos.part_of_speech_detail_2
                    && part_of_speech_detail_3 == pos.part_of_speech_detail_3
                    && context_id == pos.context_id
            })
            .ok_or_else(|| D::Error::custom("could not determine `word_type`"))?;

        let this = Self::new(&surface, pronunciation, accent_type, *word_type, priority)
            .map_err(D::Error::custom)?;

        if let Some(mora_count) = mora_count {
            if this.mora_count != mora_count {
                return Err(D::Error::custom("wrong value for `mora_count`"));
            }
        }

        Ok(this)
    }
}

impl Serialize for UserDictWord {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let Self {
            surface,
            pronunciation,
            accent_type,
            word_type,
            priority,
            mora_count,
        } = self;
        let priority = *priority;
        let accent_type = *accent_type;
        let mora_count = Some(*mora_count);

        let PartOfSpeechDetail {
            part_of_speech,
            part_of_speech_detail_1,
            part_of_speech_detail_2,
            part_of_speech_detail_3,
            context_id,
            ..
        } = PART_OF_SPEECH_DETAIL[word_type];

        SerdeRepr::<&str> {
            surface,
            priority,
            context_id,
            part_of_speech,
            part_of_speech_detail_1,
            part_of_speech_detail_2,
            part_of_speech_detail_3,
            inflectional_type: "*",
            inflectional_form: "*",
            stem: "*",
            yomi: pronunciation,
            pronunciation,
            accent_type,
            mora_count,
            accent_associative_rule: "*",
        }
        .serialize(serializer)
    }
}

/// [`UserDictWord`]のビルダー。
#[derive(Debug)]
pub struct UserDictWordBuilder {
    word_type: UserDictWordType,
    priority: u32,
}

#[expect(clippy::enum_variant_names, reason = "特に理由はないので正されるべき")] // FIXME
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

pub const DEFAULT_WORD_TYPE: UserDictWordType = UserDictWordType::CommonNoun;
pub const DEFAULT_PRIORITY: u32 = 5;

static PRONUNCIATION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[ァ-ヴー]+$").unwrap());

impl UserDictWord {
    #[cfg_attr(doc, doc(alias = "voicevox_user_dict_word_make"))]
    pub fn builder() -> UserDictWordBuilder {
        Default::default()
    }

    fn new(
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
            surface: hankaku_zenkaku::to_zenkaku(surface),
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
    let mora_count = katakana::count_moras(pronunciation);

    if accent_type > mora_count {
        return Err(InvalidWordError::InvalidAccentType(
            accent_type,
            ..=mora_count,
        ));
    }

    Ok(mora_count)
}

impl UserDictWordBuilder {
    /// 単語の種類。
    pub fn word_type(self, word_type: UserDictWordType) -> Self {
        Self { word_type, ..self }
    }

    /// 単語の優先度。
    pub fn priority(self, priority: u32) -> Self {
        Self { priority, ..self }
    }

    /// [`UserDictWord`]をコンストラクトする。
    pub fn build(
        self,
        surface: &str,
        pronunciation: String,
        accent_type: usize,
    ) -> crate::Result<UserDictWord> {
        UserDictWord::new(
            surface,
            pronunciation,
            accent_type,
            self.word_type,
            self.priority,
        )
    }
}

impl Default for UserDictWordBuilder {
    fn default() -> Self {
        Self {
            word_type: DEFAULT_WORD_TYPE,
            priority: DEFAULT_PRIORITY,
        }
    }
}

/// ユーザー辞書の単語の種類。
#[cfg_attr(doc, doc(alias = "VoicevoxUserDictWordType"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
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
    #[doc(hidden)]
    __NonExhaustive,
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

#[derive(Deserialize, Serialize)]
struct SerdeRepr<S> {
    surface: S,
    priority: u32,
    #[serde(default = "default_context_id")]
    context_id: i32,
    part_of_speech: S,
    part_of_speech_detail_1: S,
    part_of_speech_detail_2: S,
    part_of_speech_detail_3: S,
    inflectional_type: S,
    inflectional_form: S,
    stem: S,
    yomi: S,
    pronunciation: S,
    accent_type: usize,
    mora_count: Option<usize>,
    accent_associative_rule: S,
}

const fn default_context_id() -> i32 {
    1348
}

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};
    use serde_json::json;

    use super::{InvalidWordError, UserDictWord, UserDictWordType};

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

    #[rstest]
    fn none_mora_count(word: UserDictWord) {
        let word1 = &word;

        let mut word2 = serde_json::to_value(word1).unwrap();
        word2["mora_count"] = json!(null);
        let word2 = serde_json::from_value::<UserDictWord>(word2).unwrap();

        assert_eq!(word2.mora_count, word1.mora_count);
    }

    #[rstest]
    fn wrong_mora_count(word: UserDictWord) {
        let mut word = serde_json::to_value(&word).unwrap();
        word["mora_count"] = json!(0);
        let err = serde_json::from_value::<UserDictWord>(word)
            .unwrap_err()
            .to_string();

        assert_eq!("wrong value for `mora_count`", err);
    }

    #[rstest]
    #[case("inflectional_type")]
    #[case("inflectional_form")]
    #[case("stem")]
    #[case("yomi")]
    #[case("accent_associative_rule")]
    fn unmodifiable_fields(word: UserDictWord, #[case] field: &str) {
        let mut word = serde_json::to_value(word).unwrap();
        word[field] = json!("_");
        serde_json::from_value::<UserDictWord>(word).unwrap_err();
    }

    #[rstest]
    fn unknown_part_of_speech(word: UserDictWord) {
        let mut word = serde_json::to_value(word).unwrap();
        word["part_of_speech"] = json!("不正な値");
        let err = serde_json::from_value::<UserDictWord>(word)
            .unwrap_err()
            .to_string();

        assert_eq!("could not determine `word_type`", err);
    }

    #[fixture]
    fn word() -> UserDictWord {
        UserDictWord::new(
            "単語",
            "ヨミ".to_owned(),
            0,
            UserDictWordType::CommonNoun,
            5,
        )
        .unwrap()
    }
}
