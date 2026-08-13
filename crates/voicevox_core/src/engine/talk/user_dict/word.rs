use std::sync::LazyLock;

use derive_more::{Binary, Into, LowerHex, Octal, UpperHex};
use duplicate::duplicate_item;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

use crate::{error::ErrorRepr, result::Result};

use super::{
    super::text::{hankaku_zenkaku, katakana},
    part_of_speech_data::{PART_OF_SPEECH_DETAIL, PartOfSpeechDetail, priority2cost},
};

/// ユーザー辞書の単語。
///
/// # Serde
///
/// [Serde]での表現はVOICEVOX
/// ENGINEに合わせた形となっており、[コンストラクタ]およびゲッターで扱う構造とは大幅に異なる。ただし今後の破壊的変更にて変わる可能性がある。[データのシリアライゼーション]を参照。
///
/// [Serde]: serde
/// [コンストラクタ]: Self::builder
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
    priority: UserDictWordPriority,

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

        if let Some(mora_count) = mora_count
            && this.mora_count != mora_count
        {
            return Err(D::Error::custom("wrong value for `mora_count`"));
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

/// ユーザー辞書における単語の優先度。
///
/// 取り得る値は`0`以上`10`以下。
#[derive(
    PartialEq,
    Eq,
    Clone,
    Copy,
    Ord,
    Hash,
    PartialOrd,
    Into,
    Serialize,
    Debug,
    derive_more::Display,
    UpperHex,
    LowerHex,
    Octal,
    Binary,
)]
pub struct UserDictWordPriority(u8);

impl UserDictWordPriority {
    /// 最小値。`0`。
    pub const MIN: Self = Self(0);

    /// 最大値。`10`。
    pub const MAX: Self = Self(10);

    /// [`u8`]から`UserDictWordPriority`をコンストラクトする。
    ///
    /// # Errors
    ///
    /// 与えられた値が`10`を超過する場合[`ErrorKind::InvalidWord`]を表すエラーを返す。
    ///
    /// [`ErrorKind::InvalidWord`]: crate::ErrorKind::InvalidWord
    pub fn new(value: u8) -> Result<Self> {
        Self::__new(value).ok_or_else(|| {
            ErrorRepr::InvalidWord(InvalidWordError::InvalidPriority {
                is_validation_of_whole_word: false,
                actual_int: value.into(),
            })
            .into()
        })
    }

    #[doc(hidden)]
    pub const fn __new(value: u8) -> Option<Self> {
        const _: () = assert!(UserDictWordPriority::MIN.0 == 0);
        if value <= Self::MAX.0 {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }

    pub(super) const fn to_index(self) -> usize {
        (Self::MAX.0 - self.0) as _
    }
}

impl Default for UserDictWordPriority {
    fn default() -> Self {
        const _: () = assert!(UserDictWordPriority::MIN.0 == 0 && UserDictWordPriority::MAX.0 >= 5);
        Self(5)
    }
}

impl TryFrom<u8> for UserDictWordPriority {
    type Error = crate::Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        Self::new(value)
    }
}

// `{i,u}128`をやるのはちょっとだけ面倒そう
#[duplicate_item(
    T;
    [ u16 ];
    [ u32 ];
    [ u64 ];
    [ usize ];
    [ i8 ];
    [ i16 ];
    [ i32 ];
    [ i64 ];
    [ isize ];
)]
impl TryFrom<T> for UserDictWordPriority {
    type Error = crate::Error;

    fn try_from(value: T) -> std::result::Result<Self, Self::Error> {
        let value = value
            .try_into()
            .map_err(|_| InvalidWordError::InvalidPriority {
                is_validation_of_whole_word: false,
                actual_int: value.into(),
            })?;
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for UserDictWordPriority {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

/// 定数から[`UserDictWordPriority`]をコンストラクトする。
///
/// ```
/// use voicevox_core::{UserDictWordPriority, user_dict_word_priority};
///
/// const _: UserDictWordPriority = user_dict_word_priority!(0);
/// const _: UserDictWordPriority = user_dict_word_priority!(5);
/// const _: UserDictWordPriority = user_dict_word_priority!(10);
/// ```
///
/// ```compile_fail
/// # use voicevox_core::{UserDictWordPriority, user_dict_word_priority};
/// #
/// const _: UserDictWordPriority = user_dict_word_priority!(11);
/// ```
#[macro_export]
macro_rules! user_dict_word_priority {
    ($value:expr $(,)?) => {{
        const PRIORITY: $crate::UserDictWordPriority =
            $crate::UserDictWordPriority::__new($value).expect("must equal or be less than 10");
        PRIORITY
    }};
}

/// [`UserDictWord`]のビルダー。
#[derive(Debug)]
pub struct UserDictWordBuilder {
    word_type: UserDictWordType,
    priority: UserDictWordPriority,
}

// FIXME: `clippy::enum_variant_names`にならって"Invalid"という接頭語を省く。
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum InvalidWordError {
    #[error("{}: 無効な発音です({_1}): {_0:?}", Self::BASE_MSG)]
    InvalidPronunciation(String, &'static str),
    #[error(
        "{prefix}優先度は{MIN}以上{MAX}以下である必要があります: {actual_int}",
        prefix = if *is_validation_of_whole_word {
            format!("{}: ", Self::BASE_MSG)
        } else {
            "".to_owned()
        },
        MIN = UserDictWordPriority::MIN,
        MAX = UserDictWordPriority::MAX
    )]
    InvalidPriority {
        // FIXME: あまりよい形には思えないのでよりよい形を考える。
        /// `UserDictWordPriority`型を形成しない他言語ラッパーでは`true`にする想定。
        is_validation_of_whole_word: bool,
        /// 実際に与えられた整数。
        actual_int: serde_json::Number,
    },
    #[error(
        "{}: 誤ったアクセント型です({1:?}の範囲から外れています): {_0}",
        Self::BASE_MSG
    )]
    InvalidAccentType(usize, std::ops::RangeToInclusive<usize>),
}

impl InvalidWordError {
    const BASE_MSG: &'static str = "ユーザー辞書の単語のバリデーションに失敗しました";
}

type InvalidWordResult<T> = std::result::Result<T, InvalidWordError>;

pub const DEFAULT_WORD_TYPE: UserDictWordType = UserDictWordType::CommonNoun;

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
        priority: UserDictWordPriority,
    ) -> Result<Self> {
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
    pub fn priority(&self) -> UserDictWordPriority {
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
    pub fn priority(self, priority: UserDictWordPriority) -> Self {
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
            priority: Default::default(),
        }
    }
}

/// ユーザー辞書の単語の種類。
///
/// # Serde
///
/// [Serde]においては各バリアント名はSCREAMING\_SNAKE\_CASEとなる。
///
/// [Serde]: serde
#[cfg_attr(doc, doc(alias = "VoicevoxUserDictWordType"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserDictWordType {
    /// 固有名詞。
    ///
    /// # Serde
    ///
    /// [Serde]においては`"PROPER_NOUN"`という値で表される。
    ///
    /// [Serde]: serde
    ProperNoun,

    /// 一般名詞。
    ///
    /// # Serde
    ///
    /// [Serde]においては`"COMMON_NOUN"`という値で表される。
    ///
    /// [Serde]: serde
    CommonNoun,

    /// 動詞。
    ///
    /// # Serde
    ///
    /// [Serde]においては`"VERB"`という値で表される。
    ///
    /// [Serde]: serde
    Verb,

    /// 形容詞。
    ///
    /// # Serde
    ///
    /// [Serde]においては`"ADJECTIVE"`という値で表される。
    ///
    /// [Serde]: serde
    Adjective,

    /// 接尾辞。
    ///
    /// # Serde
    ///
    /// [Serde]においては`"SUFFIX"`という値で表される。
    ///
    /// [Serde]: serde
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
    priority: UserDictWordPriority,
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

    use super::{InvalidWordError, UserDictWord, UserDictWordPriority, UserDictWordType};

    #[rstest]
    fn to_mecab_format_works() {
        // テストの期待値は、VOICEVOX Engineが一時的に出力するcsvの内容を使用した。
        let word = UserDictWord::new(
            "単語",
            "ヨミ".to_string(),
            0,
            UserDictWordType::ProperNoun,
            user_dict_word_priority!(5),
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

    #[test]
    fn priority_validation_works() {
        for n in 0i64..=10 {
            UserDictWordPriority::try_from(n).unwrap();
        }
        for n in [-10000i64, -1, 11, 255, 256, 10000] {
            UserDictWordPriority::try_from(n).unwrap_err();
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
            user_dict_word_priority!(5),
        )
        .unwrap()
    }
}
