use std::{
    convert::{self, Infallible},
    fmt,
    str::FromStr,
    sync::Arc,
};

use derive_more::AsRef;
use duplicate::duplicate_item;
use num_traits::ToPrimitive as _;
use pastey::paste;
use serde::{
    de::{self, Unexpected},
    Deserialize, Deserializer, Serialize,
};
use typed_floats::{NonNaNFinite, PositiveFinite};
use typeshare::U53;

use crate::{
    error::{InvalidQueryError, InvalidQueryErrorSource},
    SamplingRate,
};

use super::super::Phoneme;

pub use self::{key::Key, optional_lyric::OptionalLyric};

/// 楽譜情報。
///
/// # Validation
///
/// この構造体は不正な状態を表現しうる。どのような状態が不正なのかについては[`validate`メソッド]を参照。この構造体を使う関数は、不正な状態に対して[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
///
/// [`Deserialize`]時には、不正な状態であるかの検証は行われない。外部からのデータが不正でないことを確かめるには、デシリアライズ後に`validate`メソッドを用いる必要がある。
///
/// ```
/// # use voicevox_core::Score;
/// # let json = r#"{ "notes": [{ "lyric": "", "frame_length": 0 }] }"#;
/// let score = serde_json::from_str::<Score>(json)?;
/// score.validate()?;
/// # anyhow::Ok(())
/// ```
///
/// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
/// [`validate`メソッド]: Self::validate
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Score {
    /// 音符のリスト。
    pub notes: Vec<Note>,
}

impl From<&'_ Score> for serde_json::Value {
    fn from(value: &'_ Score) -> Self {
        serde_json::to_value(value).expect("all of the fields should be always serializable")
    }
}

/// 音符ごとの情報。
///
/// # Validation
///
/// この構造体は不正な状態を表現しうる。どのような状態が不正なのかについては[`validate`メソッド]を参照。この構造体を使う関数は、不正な状態に対して[`ErrorKind::InvalidQuery`]を表わすエラーを返す。
///
/// [`Deserialize`]時には、不正な状態であるかの検証は行われない。外部からのデータが不正でないことを確かめるには、デシリアライズ後に`validate`メソッドを用いる必要がある。
///
/// ```
/// # use voicevox_core::Note;
/// # let json = r#"{ "lyric": "", "frame_length": 0 }"#;
/// let note = serde_json::from_str::<Note>(json)?;
/// note.validate()?;
/// # anyhow::Ok(())
/// ```
///
/// [`ErrorKind::InvalidQuery`]: crate::ErrorKind::InvalidQuery
/// [`validate`メソッド]: Self::validate
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Note {
    /// ID。
    pub id: Option<NoteId>,

    /// 音階。
    pub key: Option<Key>,

    /// 歌詞。
    pub lyric: OptionalLyric,

    /// 音符のフレーム長。
    pub frame_length: U53,
}

impl From<&'_ Note> for serde_json::Value {
    fn from(value: &'_ Note) -> Self {
        serde_json::to_value(value).expect("all of the fields should be always serializable")
    }
}

/// 定数から[`Key`]をコンストラクトする。
///
/// ```
/// use voicevox_core::{key, Key};
///
/// const C4: Key = key!(60);
/// const D4: Key = key!(60 + 2);
/// ```
///
/// ```compile_fail
/// # use voicevox_core::{key, Key};
/// #
/// const _: Key = key!(-1);
/// ```
///
/// ```compile_fail
/// # use voicevox_core::{key, Key};
/// #
/// const _: Key = key!(128);
/// ```
#[macro_export]
macro_rules! key {
    ($value:expr $(,)?) => {{
        const KEY: $crate::Key = $crate::Key::__new($value).expect("value must inside `0..=127`");
        KEY
    }};
}

impl Key {
    const NAME: &str = "音階";

    const fn saturating_from(n: u8) -> Self {
        assert!(Self::MIN.get() == 0);
        if let Some(ret) = Self::__new(n) {
            ret
        } else {
            Self::MAX
        }
    }
}

impl FromStr for Key {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n = s
            .parse::<serde_json::Number>()
            .map_err(|source| InvalidQueryError {
                what: Self::NAME,
                value: Some(Box::new(s.to_owned())),
                source: Some(InvalidQueryErrorSource::NotInteger(source)),
            })?;

        n.as_u64()
            .and_then(|n| Self::__new(n.try_into().ok()?))
            .ok_or_else(|| {
                InvalidQueryError {
                    what: Self::NAME,
                    value: Some(Box::new(n)),
                    source: Some(InvalidQueryErrorSource::OutOfRangeKeyValue),
                }
                .into()
            })
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        return deserializer.deserialize_u8(Visitor);

        struct Visitor;

        impl de::Visitor<'_> for Visitor {
            type Value = Key;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(fmt, "an integer inside `0..=127`")
            }

            #[duplicate_item(
                T unexpected ;
                [ u8 ] [ |v| Unexpected::Unsigned(u64::from(v)) ];
                [ u16 ] [ |v| Unexpected::Unsigned(u64::from(v)) ];
                [ u32 ] [ |v| Unexpected::Unsigned(u64::from(v)) ];
                [ u64 ] [ |v| Unexpected::Unsigned(u64::from(v)) ];
                [ i8 ] [ |v| Unexpected::Signed(i64::from(v)) ];
                [ i16 ] [ |v| Unexpected::Signed(i64::from(v)) ];
                [ i32 ] [ |v| Unexpected::Signed(i64::from(v)) ];
                [ i64 ] [ |v| Unexpected::Signed(i64::from(v)) ];
            )]
            paste! {
                fn [<visit_ T>] <E>(self, v: T) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    v.to_u8()
                        .and_then(Key::__new)
                        .ok_or_else(|| de::Error::invalid_value((unexpected)(v), &self))
                }
            }
        }
    }
}

#[duplicate_item(
    T;
    [ u8 ];
    [ u16 ];
    [ u32 ];
    [ u64 ];
    [ u128 ];
    [ usize ];
    [ i8 ];
    [ i16 ];
    [ i32 ];
    [ i64 ];
    [ i128 ];
    [ isize ];
)]
impl TryFrom<T> for Key {
    type Error = crate::Error;

    fn try_from(n: T) -> Result<Self, Self::Error> {
        n.to_u8().and_then(Self::__new).ok_or_else(|| {
            InvalidQueryError {
                what: Self::NAME,
                value: Some(Box::new(n)),
                source: Some(InvalidQueryErrorSource::OutOfRangeKeyValue),
            }
            .into()
        })
    }
}

#[duplicate_item(
    T from_u8;
    [ u16 ] [ Into::into ];
    [ u32 ] [ Into::into ];
    [ u64 ] [ Into::into ];
    [ u128 ] [ Into::into ];
    [ usize ] [ Into::into ];
    [ i8 ] [ |n| TryFrom::try_from(n).expect("should be inside `0..=127`") ];
    [ i16 ] [ Into::into ];
    [ i32 ] [ Into::into ];
    [ i64 ] [ Into::into ];
    [ i128 ] [ Into::into ];
    [ isize ] [ Into::into ];
)]
impl From<Key> for T {
    fn from(key: Key) -> Self {
        (from_u8)(u8::from(key))
    }
}

impl FromStr for OptionalLyric {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).map_err(|()| {
            InvalidQueryError {
                what: "歌詞",
                value: Some(Box::new(s.to_owned())),
                source: None,
            }
            .into()
        })
    }
}

impl From<&'_ OptionalLyric> for serde_json::Value {
    fn from(value: &'_ OptionalLyric) -> Self {
        serde_json::to_value(value).expect("should be always serializable")
    }
}

impl<'de> Deserialize<'de> for OptionalLyric {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        return deserializer.deserialize_str(Visitor);

        struct Visitor;

        impl de::Visitor<'_> for Visitor {
            type Value = OptionalLyric;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(fmt, "a string that represents zero or one mora kana")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                OptionalLyric::new(s)
                    .map_err(|()| de::Error::invalid_value(Unexpected::Str(s), &self))
            }
        }
    }
}

/// フレームごとの音声合成用のクエリ。
///
/// # Serde
///
/// [Serde]においてはフィールド名はsnake\_caseの形ではなく、VOICEVOX
/// ENGINEに合わせる形でcamelCaseになっている。ただし今後の破壊的変更にて変わる可能性がある。[データのシリアライゼーション]を参照。
///
/// [Serde]: serde
/// [データのシリアライゼーション]: https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FrameAudioQuery {
    /// フレームごとの基本周波数。
    pub f0: Vec<PositiveFinite<f32>>,

    /// フレームごとの音量。
    pub volume: Vec<NonNaNFinite<f32>>,

    /// 音素のリスト。
    pub phonemes: Vec<FramePhoneme>,

    /// 全体の音量。
    ///
    /// # Serde
    ///
    /// [Serde]においては`volumeScale`という名前で扱われる。
    ///
    /// [Serde]: serde
    pub volume_scale: PositiveFinite<f32>,

    /// 音声データの出力サンプリングレート。
    ///
    /// # Serde
    ///
    /// [Serde]においては`outputSamplingRate`という名前で扱われる。
    ///
    /// [Serde]: serde
    pub output_sampling_rate: SamplingRate,

    /// 音声データをステレオ出力するか否か。
    ///
    /// # Serde
    ///
    /// [Serde]においては`outputStereo`という名前で扱われる。
    ///
    /// [Serde]: serde
    pub output_stereo: bool,
}

impl From<&'_ FrameAudioQuery> for serde_json::Value {
    fn from(value: &'_ FrameAudioQuery) -> Self {
        serde_json::to_value(value).expect("all of the fields should be always serializable")
    }
}

/// 音素の情報。
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct FramePhoneme {
    /// 音素。
    pub phoneme: Phoneme,

    /// 音素のフレーム長。
    pub frame_length: U53,

    /// 音符のID。
    pub note_id: Option<NoteId>,
}

impl From<&'_ FramePhoneme> for serde_json::Value {
    fn from(value: &'_ FramePhoneme) -> Self {
        serde_json::to_value(value).expect("all of the fields should be always serializable")
    }
}

/// 音符のID。
#[derive(
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    derive_more::Display,
    AsRef,
    Deserialize,
    Serialize,
)]
#[as_ref(str)]
pub struct NoteId(pub Arc<str>);

impl FromStr for NoteId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.into()))
    }
}

#[duplicate_item(
    T f;
    [ Arc<str> ] [ convert::identity ];
    [ &'_ str ] [ Into::into ];
    [ &'_ mut str ] [ Into::into ];
    [ String ] [ Into::into ];
)]
impl From<T> for NoteId {
    fn from(s: T) -> Self {
        Self(f(s))
    }
}

impl From<&'_ NoteId> for serde_json::Value {
    fn from(value: &'_ NoteId) -> Self {
        serde_json::to_value(value).expect("should be always serializable")
    }
}

mod key {
    use derive_more::{Binary, Into, LowerHex, Octal, UpperHex};
    use serde::Serialize;

    /// 音階。
    ///
    /// 取り得る値は`0`以上`127`以下。
    ///
    /// [`TryFrom`]、[`FromStr`]、[`Deserialize`]、[`key!`]からコンストラクトできる。
    ///
    /// [`FromStr`]: std::str::FromStr
    /// [`Deserialize`]: serde::Deserialize
    #[derive(
        PartialEq,
        Eq,
        Clone,
        Copy,
        Ord,
        Hash,
        PartialOrd,
        Debug,
        derive_more::Display,
        UpperHex,
        LowerHex,
        Octal,
        Binary,
        Into,
        Serialize,
    )]
    #[display("{_0}")]
    pub struct Key(
        /// # Invariant
        ///
        /// This must be inside `0..=127`.
        u8,
    );

    impl Key {
        /// 最小値。`0`。
        pub const MIN: Self = Self(0);

        /// 最大値。`127`。
        pub const MAX: Self = Self(127);

        /// ビット長。
        pub const BITS: u32 = u8::BITS - Key::MAX.0.leading_zeros();

        #[doc(hidden)]
        pub const fn __new(n: u8) -> Option<Self> {
            const _: () = assert!(Key::MIN.0 == 0);
            if n <= Self::MAX.0 {
                Some(Self(n))
            } else {
                None
            }
        }

        pub const fn get(self) -> u8 {
            self.0
        }

        /// [`{integer}::strict_add`]と同じことを行う。
        ///
        /// # Panics
        ///
        /// ## Overflow behavior
        ///
        /// オーバーフローするならパニックする。
        ///
        /// # Examples
        ///
        /// ```
        /// use voicevox_core::key;
        ///
        /// assert_eq!(key!(61), key!(60).strict_add(1));
        /// ```
        ///
        /// ```should_panic
        /// # use voicevox_core::Key;
        /// #
        /// Key::MAX.strict_add(1);
        /// ```
        ///
        /// [`{integer}::strict_add`]: u8::strict_add
        #[must_use = "same reason as `{integer}::strict_add`"]
        #[track_caller]
        pub const fn strict_add(self, rhs: u8) -> Self {
            if let Some(n) = self.0.checked_add(rhs) {
                if let Some(ret) = Self::__new(n) {
                    return ret;
                }
            }
            panic!("attempt to add with overflow");
        }

        /// [`u{n}::strict_add_signed`]と同じことを行う。
        ///
        /// # Panics
        ///
        /// ## Overflow behavior
        ///
        /// オーバーフローするならパニックする。
        ///
        /// # Examples
        ///
        /// ```
        /// use voicevox_core::key;
        ///
        /// assert_eq!(key!(59), key!(60).strict_add_signed(-1));
        /// ```
        ///
        /// ```should_panic
        /// # use voicevox_core::Key;
        /// #
        /// Key::MAX.strict_add_signed(1);
        /// ```
        ///
        /// ```should_panic
        /// # use voicevox_core::key;
        /// #
        /// key!(0).strict_add_signed(-1);
        /// ```
        ///
        /// [`u{n}::strict_add_signed`]: u8::strict_add_signed
        #[must_use = "same reason as `u{n}::strict_add_signed`"]
        #[track_caller]
        pub const fn strict_add_signed(self, rhs: i8) -> Self {
            if let Some(n) = self.0.checked_add_signed(rhs) {
                if let Some(ret) = Self::__new(n) {
                    return ret;
                }
            }
            panic!("attempt to add with overflow");
        }

        /// [`{integer}::strict_sub`]と同じことを行う。
        ///
        /// # Panics
        ///
        /// ## Overflow behavior
        ///
        /// オーバーフローするならパニックする。
        ///
        /// # Examples
        ///
        /// ```
        /// use voicevox_core::key;
        ///
        /// assert_eq!(key!(59), key!(60).strict_sub(1));
        /// ```
        ///
        /// ```should_panic
        /// # use voicevox_core::key;
        /// #
        /// key!(0).strict_sub(1);
        /// ```
        ///
        /// [`{integer}::strict_sub`]: u8::strict_sub
        #[must_use = "same reason as `{integer}::strict_sub`"]
        #[track_caller]
        pub const fn strict_sub(self, rhs: u8) -> Self {
            if let Some(n) = self.0.checked_sub(rhs) {
                if let Some(ret) = Self::__new(n) {
                    return ret;
                }
            }
            panic!("attempt to subtract with overflow");
        }

        // TODO: Rust 1.90以降になったら`strict_sub_signed`も追加する。

        /// [`{integer}::saturating_add`]と同じことを行う。
        ///
        /// # Examples
        ///
        /// ```
        /// use voicevox_core::key;
        ///
        /// assert_eq!(key!(61), key!(60).saturating_add(1));
        /// assert_eq!(key!(127), key!(126).saturating_add(2));
        /// ```
        ///
        /// [`{integer}::saturating_add`]: u8::saturating_add
        #[must_use = "same reason as `{integer}::saturating_add`"]
        pub const fn saturating_add(self, rhs: u8) -> Self {
            Self::saturating_from(self.0.saturating_add(rhs))
        }

        /// [`u{n}::saturating_add_signed`]と同じことを行う。
        ///
        /// # Examples
        ///
        /// ```
        /// use voicevox_core::key;
        ///
        /// assert_eq!(key!(61), key!(60).saturating_add_signed(1));
        /// assert_eq!(key!(0), key!(1).saturating_add_signed(-2));
        /// assert_eq!(key!(127), key!(126).saturating_add_signed(2));
        /// ```
        ///
        /// [`u{n}::saturating_add_signed`]: u8::saturating_add_signed
        #[must_use = "same reason as `u{n}::saturating_add_signed`"]
        pub const fn saturating_add_signed(self, rhs: i8) -> Self {
            Self::saturating_from(self.0.saturating_add_signed(rhs))
        }

        /// [`{integer}::saturating_sub`]と同じことを行う。
        ///
        /// # Examples
        ///
        /// ```
        /// use voicevox_core::key;
        ///
        /// assert_eq!(key!(59), key!(60).saturating_sub(1));
        /// assert_eq!(key!(0), key!(1).saturating_sub(2));
        /// ```
        ///
        /// [`{integer}::saturating_sub`]: u8::saturating_sub
        #[must_use = "same reason as `{integer}::saturating_sub`"]
        pub const fn saturating_sub(self, rhs: u8) -> Self {
            Self::__new(self.0.saturating_sub(rhs)).expect("should be in the range")
        }

        // TODO: Rust 1.90以降になったら`saturating_sub_signed`も追加する。
    }
}

mod optional_lyric {
    use arrayvec::ArrayVec;
    use derive_more::AsRef;
    use serde_with::SerializeDisplay;
    use smol_str::SmolStr;

    use super::super::super::{
        acoustic_feature_extractor::{NonPauBaseVowel, OptionalConsonant},
        mora_mappings::MORA_KANA_TO_MORA_PHONEMES,
    };

    /// 音符の歌詞。空文字列は[無音]。
    ///
    /// # Examples
    ///
    /// ```
    /// # use voicevox_core::OptionalLyric;
    /// #
    /// "ア".parse::<OptionalLyric>()?;
    /// "ヴォ".parse::<OptionalLyric>()?;
    /// "ん".parse::<OptionalLyric>()?; // 平仮名
    /// "".parse::<OptionalLyric>()?; // 無音
    ///
    /// "アア".parse::<OptionalLyric>().unwrap_err(); // 複数モーラは現状非対応
    /// # anyhow::Ok(())
    /// ```
    ///
    /// [無音]: crate::Phoneme::MorablePau
    #[derive(
        Clone,
        Default,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Debug,
        derive_more::Display,
        AsRef,
        SerializeDisplay,
    )]
    #[display("{text}")]
    pub struct OptionalLyric {
        /// # Invariant
        ///
        /// `phonemes` must come from this.
        #[as_ref(str)]
        text: SmolStr,

        /// # Invariant
        ///
        /// This must come from `text`.
        pub(super) phonemes: ArrayVec<(OptionalConsonant, NonPauBaseVowel), 1>,
    }

    impl OptionalLyric {
        /// [無音]。
        ///
        /// ```
        /// # use voicevox_core::OptionalLyric;
        /// #
        /// assert_eq!(OptionalLyric::default(), OptionalLyric::PAU);
        /// assert_eq!("", OptionalLyric::PAU.as_ref());
        /// ```
        ///
        /// [無音]: crate::Phoneme::MorablePau
        pub const PAU: Self = Self {
            text: SmolStr::new_static(""),
            phonemes: ArrayVec::new_const(),
        };

        pub(super) fn new(text: &str) -> Result<Self, ()> {
            if text.is_empty() {
                return Ok(Self::default());
            }

            let mora_kana = hira_to_kana(text).parse().map_err(|_| ())?;

            Ok(Self {
                text: text.into(),
                phonemes: [MORA_KANA_TO_MORA_PHONEMES[mora_kana]].into(),
            })
        }

        pub(in super::super) fn phonemes(
            &self,
        ) -> &ArrayVec<(OptionalConsonant, NonPauBaseVowel), 1> {
            &self.phonemes
        }
    }

    pub(super) fn hira_to_kana(s: &str) -> SmolStr {
        s.chars()
            .map(|c| match c {
                'ぁ'..='ゔ' => (u32::from(c) + 96).try_into().expect("should be OK"),
                c => c,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::cmp;

    use itertools::iproduct;
    use rstest::rstest;

    use super::key::Key;

    #[test]
    fn key_new_works() {
        for n in 0..=u8::MAX {
            const MIN: u8 = Key::MIN.get();
            const MAX: u8 = Key::MAX.get();
            match n {
                MIN..=MAX => assert!(Key::__new(n).is_some()),
                n => assert!(Key::__new(n).is_none()),
            }
        }
    }

    #[test]
    fn key_strict_add_works() {
        for (lhs, rhs) in iproduct!(Key::MIN.get()..=Key::MAX.get(), 0..=u8::MAX) {
            let lhs = Key::__new(lhs).unwrap();
            // TODO: Rust 2024にしたらlet chainにする
            if let Some(sum) = lhs.get().checked_add(rhs) {
                if sum <= Key::MAX.get() {
                    assert_eq!(sum, lhs.strict_add(rhs).get());
                }
            }
        }
    }

    #[test]
    fn key_strict_add_signed_works() {
        for (lhs, rhs) in iproduct!(Key::MIN.get()..=Key::MAX.get(), i8::MIN..=i8::MAX) {
            let lhs = Key::__new(lhs).unwrap();
            // TODO: Rust 2024にしたらlet chainにする
            if let Some(sum) = lhs.get().checked_add_signed(rhs) {
                if sum <= Key::MAX.get() {
                    assert_eq!(sum, lhs.strict_add_signed(rhs).get());
                }
            }
        }
    }

    #[test]
    fn key_strict_sub_works() {
        for (lhs, rhs) in iproduct!(Key::MIN.get()..=Key::MAX.get(), 0..=u8::MAX) {
            let lhs = Key::__new(lhs).unwrap();
            // TODO: Rust 2024にしたらlet chainにする
            if let Some(sum) = lhs.get().checked_sub(rhs) {
                if sum <= Key::MAX.get() {
                    assert_eq!(sum, lhs.strict_sub(rhs).get());
                }
            }
        }
    }

    #[test]
    fn key_saturating_add_works() {
        for (lhs, rhs) in iproduct!(Key::MIN.get()..=Key::MAX.get(), 0..=u8::MAX) {
            let lhs = Key::__new(lhs).unwrap();
            assert_eq!(
                cmp::min(lhs.get().saturating_add(rhs), Key::MAX.get()),
                lhs.saturating_add(rhs).get(),
            );
        }
    }

    #[test]
    fn key_saturating_add_signed_works() {
        for (lhs, rhs) in iproduct!(Key::MIN.get()..=Key::MAX.get(), i8::MIN..=i8::MAX) {
            let lhs = Key::__new(lhs).unwrap();
            assert_eq!(
                cmp::min(lhs.get().saturating_add_signed(rhs), Key::MAX.get()),
                lhs.saturating_add_signed(rhs).get(),
            );
        }
    }

    #[test]
    fn key_saturating_sub_works() {
        for (lhs, rhs) in iproduct!(Key::MIN.get()..=Key::MAX.get(), 0..=u8::MAX) {
            let lhs = Key::__new(lhs).unwrap();
            assert_eq!(
                cmp::min(lhs.get().saturating_sub(rhs), Key::MAX.get()),
                lhs.saturating_sub(rhs).get(),
            );
        }
    }

    #[rstest]
    #[case("ァ", "ァ")]
    #[case("ァ", "ぁ")]
    #[case("ヴ", "ゔ")]
    fn hira_to_kana_works(#[case] expected: &str, #[case] input: &str) {
        assert_eq!(expected, super::optional_lyric::hira_to_kana(input));
    }

    #[test]
    fn hira_to_kana_should_not_fail() {
        for c in 'ぁ'..='ゔ' {
            super::optional_lyric::hira_to_kana(&c.to_string());
        }
    }
}
