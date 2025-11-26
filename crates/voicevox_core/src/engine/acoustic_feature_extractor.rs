use std::str::FromStr;

use bytemuck::{checked::CheckedCastError, CheckedBitPattern, Contiguous, NoUninit};
use duplicate::duplicate_item;
use strum::EnumCount;

use self::sil::Sil;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, derive_more::Display)]
pub(crate) enum Phoneme {
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

impl FromStr for Phoneme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(sil) = s.parse() {
            Ok(Self::Sil(sil))
        } else {
            match s {
                "pau" => Ok(Self::MorablePau),
                "A" => Ok(Self::UnvoicedVowelA),
                "E" => Ok(Self::UnvoicedVowelE),
                "I" => Ok(Self::UnvoicedVowelI),
                "N" => Ok(Self::MorableN),
                "O" => Ok(Self::UnvoicedVowelO),
                "U" => Ok(Self::UnvoicedVowelU),
                "a" => Ok(Self::VoicedVowelA),
                "b" => Ok(Self::ConsonantB),
                "by" => Ok(Self::ConsonantBy),
                "ch" => Ok(Self::ConsonantCh),
                "cl" => Ok(Self::MorableCl),
                "d" => Ok(Self::ConsonantD),
                "dy" => Ok(Self::ConsonantDy),
                "e" => Ok(Self::VoicedVowelE),
                "f" => Ok(Self::ConsonantF),
                "g" => Ok(Self::ConsonantG),
                "gw" => Ok(Self::ConsonantGw),
                "gy" => Ok(Self::ConsonantGy),
                "h" => Ok(Self::ConsonantH),
                "hy" => Ok(Self::ConsonantHy),
                "i" => Ok(Self::VoicedVowelI),
                "j" => Ok(Self::ConsonantJ),
                "k" => Ok(Self::ConsonantK),
                "kw" => Ok(Self::ConsonantKw),
                "ky" => Ok(Self::ConsonantKy),
                "m" => Ok(Self::ConsonantM),
                "my" => Ok(Self::ConsonantMy),
                "n" => Ok(Self::ConsonantN),
                "ny" => Ok(Self::ConsonantNy),
                "o" => Ok(Self::VoicedVowelO),
                "p" => Ok(Self::ConsonantP),
                "py" => Ok(Self::ConsonantPy),
                "r" => Ok(Self::ConsonantR),
                "ry" => Ok(Self::ConsonantRy),
                "s" => Ok(Self::ConsonantS),
                "sh" => Ok(Self::ConsonantSh),
                "t" => Ok(Self::ConsonantT),
                "ts" => Ok(Self::ConsonantTs),
                "ty" => Ok(Self::ConsonantTy),
                "u" => Ok(Self::VoicedVowelU),
                "v" => Ok(Self::ConsonantV),
                "w" => Ok(Self::ConsonantW),
                "y" => Ok(Self::ConsonantY),
                "z" => Ok(Self::ConsonantZ),
                s => Err(format!("invalid phoneme: {s:?}")),
            }
        }
    }
}

/// 音素IDのうち、`-1` ([`OptionalConsonant::None`])を除いたもの。
#[derive(Clone, Copy, Contiguous, NoUninit, EnumCount)]
#[cfg_attr(test, derive(PartialEq, Debug, strum::EnumIter))]
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

impl PhonemeCode {
    pub(crate) const fn num_phoneme() -> usize {
        Self::COUNT
    }

    const fn space_phoneme() -> Self {
        Self::MorablePau
    }
}

impl From<Phoneme> for PhonemeCode {
    fn from(phoneme: Phoneme) -> Self {
        macro_rules! convert {
            ($($variant:ident),* $(,)?) => {
                match phoneme {
                    $(Phoneme::$variant => Self::$variant,)*
                    Phoneme::Sil(_) => Self::space_phoneme(),
                }
            };
        }

        convert!(
            MorablePau,
            UnvoicedVowelA,
            UnvoicedVowelE,
            UnvoicedVowelI,
            MorableN,
            UnvoicedVowelO,
            UnvoicedVowelU,
            VoicedVowelA,
            ConsonantB,
            ConsonantBy,
            ConsonantCh,
            MorableCl,
            ConsonantD,
            ConsonantDy,
            VoicedVowelE,
            ConsonantF,
            ConsonantG,
            ConsonantGw,
            ConsonantGy,
            ConsonantH,
            ConsonantHy,
            VoicedVowelI,
            ConsonantJ,
            ConsonantK,
            ConsonantKw,
            ConsonantKy,
            ConsonantM,
            ConsonantMy,
            ConsonantN,
            ConsonantNy,
            VoicedVowelO,
            ConsonantP,
            ConsonantPy,
            ConsonantR,
            ConsonantRy,
            ConsonantS,
            ConsonantSh,
            ConsonantT,
            ConsonantTs,
            ConsonantTy,
            VoicedVowelU,
            ConsonantV,
            ConsonantW,
            ConsonantY,
            ConsonantZ,
        )
    }
}

impl From<PhonemeCode> for usize {
    fn from(phoneme: PhonemeCode) -> Self {
        const _: () =
            assert!(0 <= PhonemeCode::MIN_VALUE && PhonemeCode::MAX_VALUE <= u16::MAX as _);
        phoneme
            .into_integer()
            .try_into()
            .expect("should be ensured by the above assertion")
    }
}

#[expect(dead_code, reason = "we use `bytemuck` to construct values instead")]
#[derive(Clone, Copy, CheckedBitPattern, NoUninit, EnumCount)]
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

#[expect(dead_code, reason = "we use `bytemuck` to construct values instead")]
#[derive(Clone, Copy, CheckedBitPattern, NoUninit, EnumCount)]
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

impl MoraTail {
    pub(crate) fn is_unvoiced(self) -> bool {
        matches!(
            self,
            Self::UnvoicedVowelA
                | Self::UnvoicedVowelI
                | Self::UnvoicedVowelU
                | Self::UnvoicedVowelE
                | Self::UnvoicedVowelO
                | Self::MorableCl
                | Self::MorablePau
        )
    }
}

#[duplicate_item(
    T;
    [ OptionalConsonant ];
    [ MoraTail ];
)]
impl TryFrom<PhonemeCode> for T {
    type Error = ();

    fn try_from(phoneme: PhonemeCode) -> Result<Self, Self::Error> {
        bytemuck::checked::try_cast(phoneme).map_err(|err| {
            assert_eq!(
                CheckedCastError::InvalidBitPattern,
                err,
                "there should be no size/alignment issues",
            );
        })
    }
}

const _: () = assert!(PhonemeCode::MIN_VALUE == 0);
const _: () = assert!(PhonemeCode::MAX_VALUE == 44);
const _: () = assert!(PhonemeCode::COUNT == 45);
const _: () = assert!(MoraTail::COUNT == 13);
const _: () = assert!(OptionalConsonant::COUNT == PhonemeCode::COUNT - MoraTail::COUNT + 1);

mod sil {
    use std::{borrow::Cow, str::FromStr};

    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, derive_more::Display)]
    pub(crate) struct Sil(
        Cow<'static, str>, // invariant: must contain "sil"
    );

    impl FromStr for Sil {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s.contains("sil") {
                Ok(Self(match s {
                    "sil" => "sil".into(),
                    s => s.to_owned().into(),
                }))
            } else {
                Err(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bytemuck::Contiguous;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use strum::IntoEnumIterator as _;

    use super::{MoraTail, OptionalConsonant, Phoneme, PhonemeCode};

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
        assert_eq!(
            format!("invalid phoneme: {s:?}"),
            s.parse::<Phoneme>().unwrap_err(),
        );
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
