use easy_ext::ext;
use enum_map::Enum;
use strum::{EnumString, IntoStaticStr};

macro_rules! phoneme {
    ("pau") => {
        crate::engine::Phoneme::MorablePau
    };
    ("A") => {
        crate::engine::Phoneme::UnvoicedVowelA
    };
    ("E") => {
        crate::engine::Phoneme::UnvoicedVowelE
    };
    ("I") => {
        crate::engine::Phoneme::UnvoicedVowelI
    };
    ("N") => {
        crate::engine::Phoneme::MorableN
    };
    ("O") => {
        crate::engine::Phoneme::UnvoicedVowelO
    };
    ("U") => {
        crate::engine::Phoneme::UnvoicedVowelU
    };
    ("a") => {
        crate::engine::Phoneme::VoicedVowelA
    };
    ("b") => {
        crate::engine::Phoneme::ConsonantB
    };
    ("by") => {
        crate::engine::Phoneme::ConsonantBy
    };
    ("ch") => {
        crate::engine::Phoneme::ConsonantCh
    };
    ("cl") => {
        crate::engine::Phoneme::MorableCl
    };
    ("d") => {
        crate::engine::Phoneme::ConsonantD
    };
    ("dy") => {
        crate::engine::Phoneme::ConsonantDy
    };
    ("e") => {
        crate::engine::Phoneme::VoicedVowelE
    };
    ("f") => {
        crate::engine::Phoneme::ConsonantF
    };
    ("g") => {
        crate::engine::Phoneme::ConsonantG
    };
    ("gw") => {
        crate::engine::Phoneme::ConsonantGw
    };
    ("gy") => {
        crate::engine::Phoneme::ConsonantGy
    };
    ("h") => {
        crate::engine::Phoneme::ConsonantH
    };
    ("hy") => {
        crate::engine::Phoneme::ConsonantHy
    };
    ("i") => {
        crate::engine::Phoneme::VoicedVowelI
    };
    ("j") => {
        crate::engine::Phoneme::ConsonantJ
    };
    ("k") => {
        crate::engine::Phoneme::ConsonantK
    };
    ("kw") => {
        crate::engine::Phoneme::ConsonantKw
    };
    ("ky") => {
        crate::engine::Phoneme::ConsonantKy
    };
    ("m") => {
        crate::engine::Phoneme::ConsonantM
    };
    ("my") => {
        crate::engine::Phoneme::ConsonantMy
    };
    ("n") => {
        crate::engine::Phoneme::ConsonantN
    };
    ("ny") => {
        crate::engine::Phoneme::ConsonantNy
    };
    ("o") => {
        crate::engine::Phoneme::VoicedVowelO
    };
    ("p") => {
        crate::engine::Phoneme::ConsonantP
    };
    ("py") => {
        crate::engine::Phoneme::ConsonantPy
    };
    ("r") => {
        crate::engine::Phoneme::ConsonantR
    };
    ("ry") => {
        crate::engine::Phoneme::ConsonantRy
    };
    ("s") => {
        crate::engine::Phoneme::ConsonantS
    };
    ("sh") => {
        crate::engine::Phoneme::ConsonantSh
    };
    ("t") => {
        crate::engine::Phoneme::ConsonantT
    };
    ("ts") => {
        crate::engine::Phoneme::ConsonantTs
    };
    ("ty") => {
        crate::engine::Phoneme::ConsonantTy
    };
    ("u") => {
        crate::engine::Phoneme::VoicedVowelU
    };
    ("v") => {
        crate::engine::Phoneme::ConsonantV
    };
    ("w") => {
        crate::engine::Phoneme::ConsonantW
    };
    ("y") => {
        crate::engine::Phoneme::ConsonantY
    };
    ("z") => {
        crate::engine::Phoneme::ConsonantZ
    };
}
pub(crate) use phoneme;

#[derive(Clone, Copy, PartialEq, Debug, Enum, EnumString, strum::Display, IntoStaticStr)]
pub(crate) enum Phoneme {
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum OjtPhoneme {
    None,
    HasSil, // TODO: https://github.com/VOICEVOX/voicevox_engine/pull/791
    HasId(Phoneme),
}

impl OjtPhoneme {
    pub(crate) const fn num_phoneme() -> usize {
        return <Phoneme as Enum>::Array::<()>::LEN;

        #[ext]
        impl<T, const N: usize> [T; N] {
            const LEN: usize = N;
        }
    }

    fn space_phoneme() -> Self {
        Self::HasId(Phoneme::MorablePau)
    }

    /// # Panics
    ///
    /// `s`が不正ならパニックする。
    pub(crate) fn new(s: &str) -> Self {
        match s {
            "" => Self::None,
            s if s.contains("sil") => Self::HasSil,
            s => Self::HasId(
                s.parse()
                    .unwrap_or_else(|_| panic!("invalid phoneme: {s:?}")),
            ),
        }
    }

    pub(crate) fn phoneme_id(&self) -> i64 {
        match self {
            Self::None => -1,
            Self::HasSil => panic!("should have been converted"),
            Self::HasId(p) => p.into_usize() as _,
        }
    }

    pub(super) fn convert(phonemes: &[OjtPhoneme]) -> Vec<OjtPhoneme> {
        let mut phonemes = phonemes.to_owned();
        // TODO: Rust 2024にしたらlet chainに戻す
        #[cfg(any())]
        __! {
        if let Some(first_phoneme) = phonemes.first_mut()
            && *first_phoneme == Self::HasSil
        {
            *first_phoneme = Self::space_phoneme();
        }
        if let Some(last_phoneme) = phonemes.last_mut()
            && *last_phoneme == Self::HasSil
        {
            *last_phoneme = Self::space_phoneme();
        }
        }
        if let Some(first_phoneme) = phonemes.first_mut() {
            if *first_phoneme == Self::HasSil {
                *first_phoneme = Self::space_phoneme();
            }
        }
        if let Some(last_phoneme) = phonemes.last_mut() {
            if *last_phoneme == Self::HasSil {
                *last_phoneme = Self::space_phoneme();
            }
        }
        phonemes
    }
}

#[cfg(test)]
mod tests {
    use enum_map::Enum as _;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::{OjtPhoneme, Phoneme};

    const STR_HELLO_HIHO: &str = "sil k o N n i ch i w a pau h i h o d e s U sil";

    fn base_hello_hiho() -> Vec<OjtPhoneme> {
        STR_HELLO_HIHO
            .split_whitespace()
            .map(OjtPhoneme::new)
            .collect()
    }

    fn ojt_hello_hiho() -> Vec<OjtPhoneme> {
        OjtPhoneme::convert(&base_hello_hiho())
    }

    #[rstest]
    #[case(1, "A")]
    #[case(14, "e")]
    #[case(26, "m")]
    #[case(38, "ts")]
    #[case(41, "v")]
    fn test_phoneme_list(#[case] index: usize, #[case] phoneme_str: &str) {
        assert_eq!(<&str>::from(Phoneme::from_usize(index)), phoneme_str);
    }

    #[rstest]
    fn test_num_phoneme_works() {
        assert_eq!(OjtPhoneme::num_phoneme(), 45);
    }

    #[rstest]
    fn test_space_phoneme_works() {
        assert_eq!(
            OjtPhoneme::space_phoneme(),
            OjtPhoneme::HasId(Phoneme::MorablePau),
        );
    }

    #[rstest]
    #[case(ojt_hello_hiho(), "pau k o N n i ch i w a pau h i h o d e s U pau")]
    fn test_convert_works(#[case] ojt_phonemes: Vec<OjtPhoneme>, #[case] expected: &str) {
        let ojt_str_hello_hiho: String = ojt_phonemes
            .iter()
            .map(|phoneme| match phoneme {
                OjtPhoneme::HasId(phoneme) => phoneme.to_string(),
                _ => panic!(),
            })
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(ojt_str_hello_hiho, expected);
    }

    #[rstest]
    #[case(ojt_hello_hiho(), 9, OjtPhoneme::new("a"), true)]
    #[case(ojt_hello_hiho(), 9, OjtPhoneme::new("k"), false)]
    fn test_ojt_phoneme_equality(
        #[case] ojt_phonemes: Vec<OjtPhoneme>,
        #[case] index: usize,
        #[case] phoneme: OjtPhoneme,
        #[case] is_equal: bool,
    ) {
        assert_eq!(ojt_phonemes[index] == phoneme, is_equal);
    }

    #[rstest]
    #[case(ojt_hello_hiho(), &[0, 23, 30, 4, 28, 21, 10, 21, 42, 7, 0, 19, 21, 19, 30, 12, 14, 35, 6, 0])]
    fn test_phoneme_id_works(#[case] ojt_phonemes: Vec<OjtPhoneme>, #[case] expected_ids: &[i64]) {
        let ojt_ids = ojt_phonemes
            .iter()
            .map(|phoneme| phoneme.phoneme_id())
            .collect::<Vec<_>>();
        assert_eq!(ojt_ids, expected_ids);
    }
}
