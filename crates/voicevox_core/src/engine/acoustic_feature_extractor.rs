pub(super) mod convert;

use bytemuck::{CheckedBitPattern, Contiguous, NoUninit};
use serde_with::SerializeDisplay;
use strum::EnumCount;

pub use self::sil::Sil;

/// 音素。
#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, derive_more::Display, SerializeDisplay,
)]
pub enum Phoneme {
    /// `pau`。
    #[display("pau")]
    MorablePau,

    /// `sil`。
    #[display("{_0}")]
    Sil(Sil),

    /// `A`。
    #[display("A")]
    UnvoicedVowelA,

    /// `E`。
    #[display("E")]
    UnvoicedVowelE,

    /// `I`。
    #[display("I")]
    UnvoicedVowelI,

    /// `N`。
    #[display("N")]
    MorableN,

    /// `O`。
    #[display("O")]
    UnvoicedVowelO,

    /// `U`。
    #[display("U")]
    UnvoicedVowelU,

    /// `a`。
    #[display("a")]
    VoicedVowelA,

    /// `b`。
    #[display("b")]
    ConsonantB,

    /// `by`。
    #[display("by")]
    ConsonantBy,

    /// `ch`。
    #[display("ch")]
    ConsonantCh,

    /// `cl`。
    #[display("cl")]
    MorableCl,

    /// `d`。
    #[display("d")]
    ConsonantD,

    /// `dy`。
    #[display("dy")]
    ConsonantDy,

    /// `e`。
    #[display("e")]
    VoicedVowelE,

    /// `f`。
    #[display("f")]
    ConsonantF,

    /// `g`。
    #[display("g")]
    ConsonantG,

    /// `gw`。
    #[display("gw")]
    ConsonantGw,

    /// `gy`。
    #[display("gy")]
    ConsonantGy,

    /// `h`。
    #[display("h")]
    ConsonantH,

    /// `hy`。
    #[display("hy")]
    ConsonantHy,

    /// `i`。
    #[display("i")]
    VoicedVowelI,

    /// `j`。
    #[display("j")]
    ConsonantJ,

    /// `k`。
    #[display("k")]
    ConsonantK,

    /// `kw`。
    #[display("kw")]
    ConsonantKw,

    /// `ky`。
    #[display("ky")]
    ConsonantKy,

    /// `m`。
    #[display("m")]
    ConsonantM,

    /// `my`。
    #[display("my")]
    ConsonantMy,

    /// `n`。
    #[display("n")]
    ConsonantN,

    /// `ny`。
    #[display("ny")]
    ConsonantNy,

    /// `o`。
    #[display("o")]
    VoicedVowelO,

    /// `p`。
    #[display("p")]
    ConsonantP,

    /// `py`。
    #[display("py")]
    ConsonantPy,

    /// `r`。
    #[display("r")]
    ConsonantR,

    /// `ry`。
    #[display("ry")]
    ConsonantRy,

    /// `s`。
    #[display("s")]
    ConsonantS,

    /// `sh`。
    #[display("sh")]
    ConsonantSh,

    /// `t`。
    #[display("t")]
    ConsonantT,

    /// `ts`。
    #[display("ts")]
    ConsonantTs,

    /// `ty`。
    #[display("ty")]
    ConsonantTy,

    /// `u`。
    #[display("u")]
    VoicedVowelU,

    /// `v`。
    #[display("v")]
    ConsonantV,

    /// `w`。
    #[display("w")]
    ConsonantW,

    /// `y`。
    #[display("y")]
    ConsonantY,

    /// `z`。
    #[display("z")]
    ConsonantZ,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, derive_more::Display)]
pub(crate) enum Consonant {
    /// `b`。
    #[display("b")]
    B,

    /// `by`。
    #[display("by")]
    By,

    /// `ch`。
    #[display("ch")]
    Ch,

    /// `d`。
    #[display("d")]
    D,

    /// `dy`。
    #[display("dy")]
    Dy,

    /// `f`。
    #[display("f")]
    F,

    /// `g`。
    #[display("g")]
    G,

    /// `gw`。
    #[display("gw")]
    Gw,

    /// `gy`。
    #[display("gy")]
    Gy,

    /// `h`。
    #[display("h")]
    H,

    /// `hy`。
    #[display("hy")]
    Hy,

    /// `j`。
    #[display("j")]
    J,

    /// `k`。
    #[display("k")]
    K,

    /// `kw`。
    #[display("kw")]
    Kw,

    /// `ky`。
    #[display("ky")]
    Ky,

    /// `m`。
    #[display("m")]
    M,

    /// `my`。
    #[display("my")]
    My,

    /// `n`。
    #[display("n")]
    N,

    /// `ny`。
    #[display("ny")]
    Ny,

    /// `p`。
    #[display("p")]
    P,

    /// `py`。
    #[display("py")]
    Py,

    /// `r`。
    #[display("r")]
    R,

    /// `ry`。
    #[display("ry")]
    Ry,

    /// `s`。
    #[display("s")]
    S,

    /// `sh`。
    #[display("sh")]
    Sh,

    /// `t`。
    #[display("t")]
    T,

    /// `ts`。
    #[display("ts")]
    Ts,

    /// `ty`。
    #[display("ty")]
    Ty,

    /// `v`。
    #[display("v")]
    V,

    /// `w`。
    #[display("w")]
    W,

    /// `y`。
    #[display("y")]
    Y,

    /// `z`。
    #[display("z")]
    Z,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, derive_more::Display)]
pub(crate) enum NonConsonant {
    /// `pau`。
    #[display("pau")]
    MorablePau,

    /// `sil`。
    #[display("{_0}")]
    Sil(Sil),

    /// `A`。
    #[display("A")]
    UnvoicedVowelA,

    /// `E`。
    #[display("E")]
    UnvoicedVowelE,

    /// `I`。
    #[display("I")]
    UnvoicedVowelI,

    /// `N`。
    #[display("N")]
    MorableN,

    /// `O`。
    #[display("O")]
    UnvoicedVowelO,

    /// `U`。
    #[display("U")]
    UnvoicedVowelU,

    /// `a`。
    #[display("a")]
    VoicedVowelA,

    /// `cl`。
    #[display("cl")]
    MorableCl,

    /// `e`。
    #[display("e")]
    VoicedVowelE,

    /// `i`。
    #[display("i")]
    VoicedVowelI,

    /// `o`。
    #[display("o")]
    VoicedVowelO,

    /// `u`。
    #[display("u")]
    VoicedVowelU,
}

/// 音素IDのうち、`-1` ([`OptionalConsonant::None`])を除いたもの。
#[derive(Clone, Copy, PartialEq, Contiguous, CheckedBitPattern, NoUninit, EnumCount)]
#[cfg_attr(test, derive(Debug, strum::EnumIter))]
#[repr(i64)]
pub(crate) enum PhonemeCode {
    //None = -1,
    MorablePau = 0,
    UnvoicedVowelA = 1,
    UnvoicedVowelE = 2,
    UnvoicedVowelI = 3,
    MorableN = 4,
    UnvoicedVowelO = 5,
    UnvoicedVowelU = 6,
    VoicedVowelA = 7,
    ConsonantB = 8,
    ConsonantBy = 9,
    ConsonantCh = 10,
    MorableCl = 11,
    ConsonantD = 12,
    ConsonantDy = 13,
    VoicedVowelE = 14,
    ConsonantF = 15,
    ConsonantG = 16,
    ConsonantGw = 17,
    ConsonantGy = 18,
    ConsonantH = 19,
    ConsonantHy = 20,
    VoicedVowelI = 21,
    ConsonantJ = 22,
    ConsonantK = 23,
    ConsonantKw = 24,
    ConsonantKy = 25,
    ConsonantM = 26,
    ConsonantMy = 27,
    ConsonantN = 28,
    ConsonantNy = 29,
    VoicedVowelO = 30,
    ConsonantP = 31,
    ConsonantPy = 32,
    ConsonantR = 33,
    ConsonantRy = 34,
    ConsonantS = 35,
    ConsonantSh = 36,
    ConsonantT = 37,
    ConsonantTs = 38,
    ConsonantTy = 39,
    VoicedVowelU = 40,
    ConsonantV = 41,
    ConsonantW = 42,
    ConsonantY = 43,
    ConsonantZ = 44,
}

#[derive(Clone, Copy, PartialEq, Debug, CheckedBitPattern, NoUninit, EnumCount)]
#[repr(i64)]
pub(crate) enum OptionalConsonant {
    None = -1,
    //MorablePau = 0,
    //UnvoicedVowelA = 1,
    //UnvoicedVowelE = 2,
    //UnvoicedVowelI = 3,
    //MorableN = 4,
    //UnvoicedVowelO = 5,
    //UnvoicedVowelU = 6,
    //VoicedVowelA = 7,
    ConsonantB = 8,
    ConsonantBy = 9,
    ConsonantCh = 10,
    //MorableCl = 11,
    ConsonantD = 12,
    ConsonantDy = 13,
    //VoicedVowelE = 14,
    ConsonantF = 15,
    ConsonantG = 16,
    ConsonantGw = 17,
    ConsonantGy = 18,
    ConsonantH = 19,
    ConsonantHy = 20,
    //VoicedVowelI = 21,
    ConsonantJ = 22,
    ConsonantK = 23,
    ConsonantKw = 24,
    ConsonantKy = 25,
    ConsonantM = 26,
    ConsonantMy = 27,
    ConsonantN = 28,
    ConsonantNy = 29,
    //VoicedVowelO = 30,
    ConsonantP = 31,
    ConsonantPy = 32,
    ConsonantR = 33,
    ConsonantRy = 34,
    ConsonantS = 35,
    ConsonantSh = 36,
    ConsonantT = 37,
    ConsonantTs = 38,
    ConsonantTy = 39,
    //VoicedVowelU = 40,
    ConsonantV = 41,
    ConsonantW = 42,
    ConsonantY = 43,
    ConsonantZ = 44,
}

#[expect(dead_code, reason = "we use `bytemuck` to construct `MorablePau`")]
#[derive(Clone, Copy, PartialEq, Debug, CheckedBitPattern, NoUninit, EnumCount)]
#[repr(i64)]
pub(crate) enum MoraTail {
    //None = -1,
    MorablePau = 0,
    UnvoicedVowelA = 1,
    UnvoicedVowelE = 2,
    UnvoicedVowelI = 3,
    MorableN = 4,
    UnvoicedVowelO = 5,
    UnvoicedVowelU = 6,
    VoicedVowelA = 7,
    //ConsonantB = 8,
    //ConsonantBy = 9,
    //ConsonantCh = 10,
    MorableCl = 11,
    //ConsonantD = 12,
    //ConsonantDy = 13,
    VoicedVowelE = 14,
    //ConsonantF = 15,
    //ConsonantG = 16,
    //ConsonantGw = 17,
    //ConsonantGy = 18,
    //ConsonantH = 19,
    //ConsonantHy = 20,
    VoicedVowelI = 21,
    //ConsonantJ = 22,
    //ConsonantK = 23,
    //ConsonantKw = 24,
    //ConsonantKy = 25,
    //ConsonantM = 26,
    //ConsonantMy = 27,
    //ConsonantN = 28,
    //ConsonantNy = 29,
    VoicedVowelO = 30,
    //ConsonantP = 31,
    //ConsonantPy = 32,
    //ConsonantR = 33,
    //ConsonantRy = 34,
    //ConsonantS = 35,
    //ConsonantSh = 36,
    //ConsonantT = 37,
    //ConsonantTs = 38,
    //ConsonantTy = 39,
    VoicedVowelU = 40,
    //ConsonantV = 41,
    //ConsonantW = 42,
    //ConsonantY = 43,
    //ConsonantZ = 44,
}

const _: () = assert!(PhonemeCode::MIN_VALUE == 0);
const _: () = assert!(PhonemeCode::MAX_VALUE == 44);
const _: () = assert!(PhonemeCode::COUNT == 45);
const _: () = assert!(MoraTail::COUNT == 13);
const _: () = assert!(OptionalConsonant::COUNT == PhonemeCode::COUNT - MoraTail::COUNT + 1);

mod sil {
    use std::borrow::Cow;

    use derive_more::AsRef;
    use serde_with::SerializeDisplay;

    /// `sil` (_silent_)。
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
        SerializeDisplay,
    )]
    #[as_ref(str)]
    pub struct Sil(
        Cow<'static, str>, // invariant: must contain "sil"
    );

    impl Sil {
        pub(super) const DEFAULT: Self = Self(Cow::Borrowed("sil"));

        pub(super) fn new(s: &str) -> Option<Self> {
            s.contains("sil").then(|| {
                Self(match s {
                    "sil" => "sil".into(),
                    s => s.to_owned().into(),
                })
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use bytemuck::Contiguous;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use strum::IntoEnumIterator as _;

    use crate::error::{ErrorRepr, InvalidQueryError};

    use super::{Consonant, MoraTail, NonConsonant, OptionalConsonant, Phoneme, PhonemeCode};

    #[test]
    fn each_phoneme_code_should_be_categorized_into_consonant_xor_mora_tail() {
        for phoneme in PhonemeCode::iter() {
            assert!(
                OptionalConsonant::try_from(phoneme).is_ok() ^ MoraTail::try_from(phoneme).is_ok()
            );
        }
    }

    const STR_HELLO_HIHO: &str = "sil k o N n i ch i w a pau h i h o d e s U sil";

    fn hello_hiho() -> Vec<PhonemeCode> {
        STR_HELLO_HIHO
            .split_whitespace()
            .map(|s| s.parse::<Phoneme>().unwrap().into())
            .collect()
    }

    #[rstest]
    #[case(0, "pau")]
    #[case(1, "A")]
    #[case(14, "e")]
    #[case(26, "m")]
    #[case(38, "ts")]
    #[case(41, "v")]
    #[case(44, "z")]
    fn test_discriminant(#[case] index: i64, #[case] phoneme_str: &str) {
        assert_eq!(
            PhonemeCode::from_integer(index).unwrap(),
            phoneme_str.parse::<Phoneme>().unwrap().into(),
        );
    }

    #[rstest]
    #[case("")]
    #[case("invalid")]
    fn test_invalid_phoneme(#[case] s: &str) {
        let err = s.parse::<Phoneme>().unwrap_err();
        let crate::Error(ErrorRepr::InvalidQuery(InvalidQueryError {
            what: "音素",
            value: Some(value),
            source: None,
        })) = err
        else {
            panic!("unexpected error: {err:?}");
        };
        assert_eq!(format!("{s:?}"), format!("{value:?}"));
    }

    #[rstest]
    #[case("")]
    #[case("invalid")]
    #[case("a")]
    fn test_invalid_consonant(#[case] s: &str) {
        let err = s.parse::<Consonant>().unwrap_err();
        let crate::Error(ErrorRepr::InvalidQuery(InvalidQueryError {
            what: "子音",
            value: Some(value),
            source: Some(_),
        })) = err
        else {
            panic!("unexpected error: {err:?}");
        };
        assert_eq!(format!("{s:?}"), format!("{value:?}"));
    }

    #[rstest]
    #[case("")]
    #[case("invalid")]
    #[case("k")]
    fn test_invalid_non_consonant(#[case] s: &str) {
        let err = s.parse::<NonConsonant>().unwrap_err();
        let crate::Error(ErrorRepr::InvalidQuery(InvalidQueryError {
            what: "非子音",
            value: Some(value),
            source: Some(_),
        })) = err
        else {
            panic!("unexpected error: {err:?}");
        };
        assert_eq!(format!("{s:?}"), format!("{value:?}"));
    }

    #[rstest]
    #[case(
        hello_hiho(),
        &[
            "pau", "k", "o", "N", "n", "i", "ch", "i", "w", "a", "pau", "h", "i", "h", "o", "d",
            "e", "s", "U", "pau",
        ],
    )]
    fn test_phoneme_into_phoneme_code_works(
        #[case] phonemes: Vec<PhonemeCode>,
        #[case] expected: &[&str],
    ) {
        let expected = expected
            .iter()
            .map(|s| s.parse::<Phoneme>().unwrap().into())
            .collect::<Vec<_>>();
        assert_eq!(phonemes, expected);
    }

    #[rstest]
    #[case(hello_hiho(), 9, "a".parse::<Phoneme>().unwrap().into(), true)]
    #[case(hello_hiho(), 9, "k".parse::<Phoneme>().unwrap().into(), false)]
    fn test_phoneme_code_equality(
        #[case] phonemes: Vec<PhonemeCode>,
        #[case] index: usize,
        #[case] phoneme: PhonemeCode,
        #[case] is_equal: bool,
    ) {
        assert_eq!(phonemes[index] == phoneme, is_equal);
    }

    #[rstest]
    #[case(hello_hiho(), &[0, 23, 30, 4, 28, 21, 10, 21, 42, 7, 0, 19, 21, 19, 30, 12, 14, 35, 6, 0])]
    fn test_phoneme_code_works_as_id(
        #[case] phonemes: Vec<PhonemeCode>,
        #[case] expected_ids: &[i64],
    ) {
        let ids = phonemes
            .into_iter()
            .map(Contiguous::into_integer)
            .collect::<Vec<_>>();
        assert_eq!(ids, expected_ids);
    }
}
