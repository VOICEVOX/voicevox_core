use bytemuck::{Contiguous, NoUninit};
use strum::EnumString;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, EnumString, strum::Display)]
pub(super) enum Phoneme {
    /// 母音モーラにおける子音部分。
    ///
    /// 通常、AudioQueryにこの値が入ることはない。またVOICEVOX
    /// ENGINEではこの値は取り扱っておらず内部エラーとなる。
    #[strum(serialize = "")]
    None,

    /// `pau`。
    #[strum(serialize = "pau")]
    MorablePau,

    /// `A`。
    #[strum(serialize = "A")]
    UnvoicedVowelA,

    /// `E`。
    #[strum(serialize = "E")]
    UnvoicedVowelE,

    /// `I`。
    #[strum(serialize = "I")]
    UnvoicedVowelI,

    /// `N`。
    #[strum(serialize = "N")]
    MorableN,

    /// `O`。
    #[strum(serialize = "O")]
    UnvoicedVowelO,

    /// `U`。
    #[strum(serialize = "U")]
    UnvoicedVowelU,

    /// `a`。
    #[strum(serialize = "a")]
    VoicedVowelA,

    /// `b`。
    #[strum(serialize = "b")]
    ConsonantB,

    /// `by`。
    #[strum(serialize = "by")]
    ConsonantBy,

    /// `ch`。
    #[strum(serialize = "ch")]
    ConsonantCh,

    /// `cl`。
    #[strum(serialize = "cl")]
    MorableCl,

    /// `d`。
    #[strum(serialize = "d")]
    ConsonantD,

    /// `dy`。
    #[strum(serialize = "dy")]
    ConsonantDy,

    /// `e`。
    #[strum(serialize = "e")]
    VoicedVowelE,

    /// `f`。
    #[strum(serialize = "f")]
    ConsonantF,

    /// `g`。
    #[strum(serialize = "g")]
    ConsonantG,

    /// `gw`。
    #[strum(serialize = "gw")]
    ConsonantGw,

    /// `gy`。
    #[strum(serialize = "gy")]
    ConsonantGy,

    /// `h`。
    #[strum(serialize = "h")]
    ConsonantH,

    /// `hy`。
    #[strum(serialize = "hy")]
    ConsonantHy,

    /// `i`。
    #[strum(serialize = "i")]
    VoicedVowelI,

    /// `j`。
    #[strum(serialize = "j")]
    ConsonantJ,

    /// `k`。
    #[strum(serialize = "k")]
    ConsonantK,

    /// `kw`。
    #[strum(serialize = "kw")]
    ConsonantKw,

    /// `ky`。
    #[strum(serialize = "ky")]
    ConsonantKy,

    /// `m`。
    #[strum(serialize = "m")]
    ConsonantM,

    /// `my`。
    #[strum(serialize = "my")]
    ConsonantMy,

    /// `n`。
    #[strum(serialize = "n")]
    ConsonantN,

    /// `ny`。
    #[strum(serialize = "ny")]
    ConsonantNy,

    /// `o`。
    #[strum(serialize = "o")]
    VoicedVowelO,

    /// `p`。
    #[strum(serialize = "p")]
    ConsonantP,

    /// `py`。
    #[strum(serialize = "py")]
    ConsonantPy,

    /// `r`。
    #[strum(serialize = "r")]
    ConsonantR,

    /// `ry`。
    #[strum(serialize = "ry")]
    ConsonantRy,

    /// `s`。
    #[strum(serialize = "s")]
    ConsonantS,

    /// `sh`。
    #[strum(serialize = "sh")]
    ConsonantSh,

    /// `t`。
    #[strum(serialize = "t")]
    ConsonantT,

    /// `ts`。
    #[strum(serialize = "ts")]
    ConsonantTs,

    /// `ty`。
    #[strum(serialize = "ty")]
    ConsonantTy,

    /// `u`。
    #[strum(serialize = "u")]
    VoicedVowelU,

    /// `v`。
    #[strum(serialize = "v")]
    ConsonantV,

    /// `w`。
    #[strum(serialize = "w")]
    ConsonantW,

    /// `y`。
    #[strum(serialize = "y")]
    ConsonantY,

    /// `z`。
    #[strum(serialize = "z")]
    ConsonantZ,

    /// `sil`。
    #[strum(serialize = "sil")]
    Sil,
}

#[derive(Clone, Copy, PartialEq, Debug, Contiguous, NoUninit)]
#[repr(i64)]
pub(crate) enum PhonemeCode {
    /// 母音モーラにおける子音部分。
    None = -1,

    /// `pau`。
    MorablePau = 0,

    /// `A`。
    UnvoicedVowelA = 1,

    /// `E`。
    UnvoicedVowelE = 2,

    /// `I`。
    UnvoicedVowelI = 3,

    /// `N`。
    MorableN = 4,

    /// `O`。
    UnvoicedVowelO = 5,

    /// `U`。
    UnvoicedVowelU = 6,

    /// `a`。
    VoicedVowelA = 7,

    /// `b`。
    ConsonantB = 8,

    /// `by`。
    ConsonantBy = 9,

    /// `ch`。
    ConsonantCh = 10,

    /// `cl`。
    MorableCl = 11,

    /// `d`。
    ConsonantD = 12,

    /// `dy`。
    ConsonantDy = 13,

    /// `e`。
    VoicedVowelE = 14,

    /// `f`。
    ConsonantF = 15,

    /// `g`。
    ConsonantG = 16,

    /// `gw`。
    ConsonantGw = 17,

    /// `gy`。
    ConsonantGy = 18,

    /// `h`。
    ConsonantH = 19,

    /// `hy`。
    ConsonantHy = 20,

    /// `i`。
    VoicedVowelI = 21,

    /// `j`。
    ConsonantJ = 22,

    /// `k`。
    ConsonantK = 23,

    /// `kw`。
    ConsonantKw = 24,

    /// `ky`。
    ConsonantKy = 25,

    /// `m`。
    ConsonantM = 26,

    /// `my`。
    ConsonantMy = 27,

    /// `n`。
    ConsonantN = 28,

    /// `ny`。
    ConsonantNy = 29,

    /// `o`。
    VoicedVowelO = 30,

    /// `p`。
    ConsonantP = 31,

    /// `py`。
    ConsonantPy = 32,

    /// `r`。
    ConsonantR = 33,

    /// `ry`。
    ConsonantRy = 34,

    /// `s`。
    ConsonantS = 35,

    /// `sh`。
    ConsonantSh = 36,

    /// `t`。
    ConsonantT = 37,

    /// `ts`。
    ConsonantTs = 38,

    /// `ty`。
    ConsonantTy = 39,

    /// `u`。
    VoicedVowelU = 40,

    /// `v`。
    ConsonantV = 41,

    /// `w`。
    ConsonantW = 42,

    /// `y`。
    ConsonantY = 43,

    /// `z`。
    ConsonantZ = 44,
}

macro_rules! phoneme_codes {
    ($($phoneme_code:tt),* $(,)?) => {
        $(crate::engine::__phoneme_code!($phoneme_code))|*
    };
}

macro_rules! __phoneme_code {
    ("pau") => {
        crate::engine::PhonemeCode::MorablePau
    };
    ("A") => {
        crate::engine::PhonemeCode::UnvoicedVowelA
    };
    ("E") => {
        crate::engine::PhonemeCode::UnvoicedVowelE
    };
    ("I") => {
        crate::engine::PhonemeCode::UnvoicedVowelI
    };
    ("N") => {
        crate::engine::PhonemeCode::MorableN
    };
    ("O") => {
        crate::engine::PhonemeCode::UnvoicedVowelO
    };
    ("U") => {
        crate::engine::PhonemeCode::UnvoicedVowelU
    };
    ("a") => {
        crate::engine::PhonemeCode::VoicedVowelA
    };
    ("b") => {
        crate::engine::PhonemeCode::ConsonantB
    };
    ("by") => {
        crate::engine::PhonemeCode::ConsonantBy
    };
    ("ch") => {
        crate::engine::PhonemeCode::ConsonantCh
    };
    ("cl") => {
        crate::engine::PhonemeCode::MorableCl
    };
    ("d") => {
        crate::engine::PhonemeCode::ConsonantD
    };
    ("dy") => {
        crate::engine::PhonemeCode::ConsonantDy
    };
    ("e") => {
        crate::engine::PhonemeCode::VoicedVowelE
    };
    ("f") => {
        crate::engine::PhonemeCode::ConsonantF
    };
    ("g") => {
        crate::engine::PhonemeCode::ConsonantG
    };
    ("gw") => {
        crate::engine::PhonemeCode::ConsonantGw
    };
    ("gy") => {
        crate::engine::PhonemeCode::ConsonantGy
    };
    ("h") => {
        crate::engine::PhonemeCode::ConsonantH
    };
    ("hy") => {
        crate::engine::PhonemeCode::ConsonantHy
    };
    ("i") => {
        crate::engine::PhonemeCode::VoicedVowelI
    };
    ("j") => {
        crate::engine::PhonemeCode::ConsonantJ
    };
    ("k") => {
        crate::engine::PhonemeCode::ConsonantK
    };
    ("kw") => {
        crate::engine::PhonemeCode::ConsonantKw
    };
    ("ky") => {
        crate::engine::PhonemeCode::ConsonantKy
    };
    ("m") => {
        crate::engine::PhonemeCode::ConsonantM
    };
    ("my") => {
        crate::engine::PhonemeCode::ConsonantMy
    };
    ("n") => {
        crate::engine::PhonemeCode::ConsonantN
    };
    ("ny") => {
        crate::engine::PhonemeCode::ConsonantNy
    };
    ("o") => {
        crate::engine::PhonemeCode::VoicedVowelO
    };
    ("p") => {
        crate::engine::PhonemeCode::ConsonantP
    };
    ("py") => {
        crate::engine::PhonemeCode::ConsonantPy
    };
    ("r") => {
        crate::engine::PhonemeCode::ConsonantR
    };
    ("ry") => {
        crate::engine::PhonemeCode::ConsonantRy
    };
    ("s") => {
        crate::engine::PhonemeCode::ConsonantS
    };
    ("sh") => {
        crate::engine::PhonemeCode::ConsonantSh
    };
    ("t") => {
        crate::engine::PhonemeCode::ConsonantT
    };
    ("ts") => {
        crate::engine::PhonemeCode::ConsonantTs
    };
    ("ty") => {
        crate::engine::PhonemeCode::ConsonantTy
    };
    ("u") => {
        crate::engine::PhonemeCode::VoicedVowelU
    };
    ("v") => {
        crate::engine::PhonemeCode::ConsonantV
    };
    ("w") => {
        crate::engine::PhonemeCode::ConsonantW
    };
    ("y") => {
        crate::engine::PhonemeCode::ConsonantY
    };
    ("z") => {
        crate::engine::PhonemeCode::ConsonantZ
    };
}

pub(crate) use {__phoneme_code, phoneme_codes};

impl PhonemeCode {
    pub(crate) const fn num_phoneme() -> usize {
        Self::MAX_VALUE as usize + 1
    }

    const fn space_phoneme() -> Self {
        Self::MorablePau
    }
}

impl From<Phoneme> for PhonemeCode {
    fn from(phoneme: Phoneme) -> Self {
        macro_rules! convert {
            ($($ident:ident),* $(,)?) => {
                match phoneme {
                    $(Phoneme::$ident => Self::$ident,)*
                    Phoneme::Sil => Self::space_phoneme(),
                }
            };
        }

        convert!(
            None,
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

#[cfg(test)]
impl From<PhonemeCode> for Phoneme {
    fn from(code: PhonemeCode) -> Self {
        macro_rules! convert {
            ($($ident:ident),* $(,)?) => {
                match code {
                    $(PhonemeCode::$ident => Self::$ident,)*
                }
            };
        }

        convert!(
            None,
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

#[cfg(test)]
mod tests {
    use bytemuck::Contiguous as _;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Phoneme, PhonemeCode};

    const STR_HELLO_HIHO: &str = "sil k o N n i ch i w a pau h i h o d e s U sil";

    fn ojt_hello_hiho() -> Vec<PhonemeCode> {
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
    fn test_phoneme_list(#[case] index: i64, #[case] phoneme_str: &str) {
        assert_eq!(
            Phoneme::from(PhonemeCode::from_integer(index).unwrap()).to_string(),
            phoneme_str,
        );
    }

    #[rstest]
    fn test_num_phoneme_works() {
        assert_eq!(PhonemeCode::num_phoneme(), 45);
    }

    #[rstest]
    fn test_space_phoneme_works() {
        assert_eq!(PhonemeCode::space_phoneme(), PhonemeCode::MorablePau);
    }

    #[rstest]
    #[case(ojt_hello_hiho(), "pau k o N n i ch i w a pau h i h o d e s U pau")]
    fn test_convert_works(#[case] ojt_phonemes: Vec<PhonemeCode>, #[case] expected: &str) {
        let ojt_str_hello_hiho: String = ojt_phonemes
            .iter()
            .map(|&code| Phoneme::from(code).to_string())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(ojt_str_hello_hiho, expected);
    }

    #[rstest]
    #[case(ojt_hello_hiho(), 9, "a".parse::<Phoneme>().unwrap().into(), true)]
    #[case(ojt_hello_hiho(), 9, "k".parse::<Phoneme>().unwrap().into(), false)]
    fn test_ojt_phoneme_equality(
        #[case] ojt_phonemes: Vec<PhonemeCode>,
        #[case] index: usize,
        #[case] phoneme: PhonemeCode,
        #[case] is_equal: bool,
    ) {
        assert_eq!(ojt_phonemes[index] == phoneme, is_equal);
    }

    #[rstest]
    #[case(ojt_hello_hiho(), &[0, 23, 30, 4, 28, 21, 10, 21, 42, 7, 0, 19, 21, 19, 30, 12, 14, 35, 6, 0])]
    fn test_phoneme_id_works(#[case] ojt_phonemes: Vec<PhonemeCode>, #[case] expected_ids: &[i64]) {
        let ojt_ids = ojt_phonemes
            .into_iter()
            .map(PhonemeCode::into_integer)
            .collect::<Vec<_>>();
        assert_eq!(ojt_ids, expected_ids);
    }
}
