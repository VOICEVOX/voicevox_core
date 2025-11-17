use std::str::FromStr;

use bytemuck::{Contiguous, NoUninit};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, derive_more::Display)]
pub(super) enum Phoneme {
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
        match s {
            s if s.contains("sil") => Ok(Self::Sil(Sil(s.to_owned()))),
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, derive_more::Display)]
pub(super) struct Sil(
    String, // invariant: must contain "sil"
);

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

macro_rules! phoneme_matches {
    ($target:expr, $($candidate:tt)|*) => {
        matches!($target, $(crate::engine::phoneme_code!($candidate))|*)
    };
}

macro_rules! phoneme_code {
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

pub(crate) use {phoneme_code, phoneme_matches};

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

#[cfg(test)]
mod tests {
    use bytemuck::Contiguous;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{Phoneme, PhonemeCode};

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
    #[should_panic(expected = "invalid phoneme: ")]
    fn test_invalid_phoneme(#[case] s: &str) {
        assert_eq!(
            format!("invalid phoneme: {s}"),
            s.parse::<Phoneme>().unwrap_err(),
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
