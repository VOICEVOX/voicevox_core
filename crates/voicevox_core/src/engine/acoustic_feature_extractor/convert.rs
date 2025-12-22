use std::{fmt, str::FromStr};

use bytemuck::{checked::CheckedCastError, Contiguous as _};
use duplicate::duplicate_item;
use pastey::paste;
use serde::{
    de::{self, Deserializer, Unexpected},
    Deserialize,
};
use strum::EnumCount as _;

use crate::error::{ErrorRepr, InvalidQueryError, InvalidQueryErrorSource};

use super::{
    sil::Sil, Consonant, MoraTail, NonConsonant, NonPauBaseVowel, OptionalConsonant, Phoneme,
    PhonemeCode,
};

macro_rules! optional_consonant {
    ("") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::None
    };
    ("b") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantB
    };
    ("by") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantBy
    };
    ("ch") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantCh
    };
    ("d") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantD
    };
    ("dy") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantDy
    };
    ("f") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantF
    };
    ("g") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantG
    };
    ("gw") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantGw
    };
    ("gy") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantGy
    };
    ("h") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantH
    };
    ("hy") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantHy
    };
    ("j") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantJ
    };
    ("k") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantK
    };
    ("kw") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantKw
    };
    ("ky") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantKy
    };
    ("m") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantM
    };
    ("my") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantMy
    };
    ("n") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantN
    };
    ("ny") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantNy
    };
    ("p") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantP
    };
    ("py") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantPy
    };
    ("r") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantR
    };
    ("ry") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantRy
    };
    ("s") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantS
    };
    ("sh") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantSh
    };
    ("t") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantT
    };
    ("ts") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantTs
    };
    ("ty") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantTy
    };
    ("v") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantV
    };
    ("w") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantW
    };
    ("y") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantY
    };
    ("z") => {
        crate::engine::acoustic_feature_extractor::OptionalConsonant::ConsonantZ
    };
}

macro_rules! non_pau_base_vowel {
    ("N") => {
        crate::engine::acoustic_feature_extractor::NonPauBaseVowel::MorableN
    };
    ("a") => {
        crate::engine::acoustic_feature_extractor::NonPauBaseVowel::VoicedVowelA
    };
    ("cl") => {
        crate::engine::acoustic_feature_extractor::NonPauBaseVowel::MorableCl
    };
    ("e") => {
        crate::engine::acoustic_feature_extractor::NonPauBaseVowel::VoicedVowelE
    };
    ("i") => {
        crate::engine::acoustic_feature_extractor::NonPauBaseVowel::VoicedVowelI
    };
    ("o") => {
        crate::engine::acoustic_feature_extractor::NonPauBaseVowel::VoicedVowelO
    };
    ("u") => {
        crate::engine::acoustic_feature_extractor::NonPauBaseVowel::VoicedVowelU
    };
}

pub(in super::super) use {non_pau_base_vowel, optional_consonant};

impl Phoneme {
    fn from_str_with_inner_error(s: &str) -> Result<Self, InvalidQueryError> {
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
                value => Err(InvalidQueryError {
                    what: "音素",
                    value: Some(Box::new(value.to_owned())),
                    source: None,
                }),
            }
        }
    }
}

impl FromStr for Phoneme {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_inner_error(s).map_err(Into::into)
    }
}

impl<'de> Deserialize<'de> for Phoneme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        return deserializer.deserialize_str(Visitor);

        struct Visitor;

        impl de::Visitor<'_> for Visitor {
            type Value = Phoneme;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(fmt, "a string that represents a phoneme")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                s.parse()
                    .map_err(|_| de::Error::invalid_value(Unexpected::Str(s), &self))
            }
        }
    }
}

impl Consonant {
    pub(in super::super) fn from_str_with_inner_error(s: &str) -> Result<Self, InvalidQueryError> {
        use self::Phoneme::*;

        let error = |source| InvalidQueryError {
            what: "子音",
            value: Some(Box::new(s.to_owned()) as _),
            source: Some(source),
        };

        let phoneme = Phoneme::from_str_with_inner_error(s)
            .map_err(|source| error(InvalidQueryErrorSource::InvalidAsSuperset(source.into())))?;

        macro_rules! convert {
            ($($variant:tt),* $(,)?) => {
                match phoneme {
                    $(paste!([<Consonant $variant>]) => Ok(Self::$variant),)*
                    MorablePau | Sil(_) | UnvoicedVowelA | UnvoicedVowelE | UnvoicedVowelI
                    | MorableN | UnvoicedVowelO | UnvoicedVowelU | VoicedVowelA | MorableCl
                    | VoicedVowelE | VoicedVowelI | VoicedVowelO | VoicedVowelU => {
                        Err(error(InvalidQueryErrorSource::IsNotConsonant))
                    }
                }
            };
        }

        convert!(
            B, By, Ch, D, Dy, F, G, Gw, Gy, H, Hy, J, K, Kw, Ky, M, My, N, Ny, P, Py, R, Ry, S, Sh,
            T, Ts, Ty, V, W, Y, Z,
        )
    }
}

impl FromStr for Consonant {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_inner_error(s).map_err(Into::into)
    }
}

impl NonConsonant {
    pub(in super::super) fn from_str_with_inner_error(s: &str) -> Result<Self, InvalidQueryError> {
        let error = |source| InvalidQueryError {
            what: "非子音",
            value: Some(Box::new(s.to_owned())),
            source: Some(source),
        };

        let phoneme = Phoneme::from_str_with_inner_error(s)
            .map_err(|source| error(InvalidQueryErrorSource::InvalidAsSuperset(source.into())))?;

        macro_rules! convert {
            ($($variant:ident),* $(,)?) => {
                match phoneme {
                    $(Phoneme::$variant => Ok(Self::$variant),)*
                    Phoneme::Sil(sil) => Ok(Self::Sil(sil)),
                    Phoneme::ConsonantB
                    | Phoneme::ConsonantBy
                    | Phoneme::ConsonantCh
                    | Phoneme::ConsonantD
                    | Phoneme::ConsonantDy
                    | Phoneme::ConsonantF
                    | Phoneme::ConsonantG
                    | Phoneme::ConsonantGw
                    | Phoneme::ConsonantGy
                    | Phoneme::ConsonantH
                    | Phoneme::ConsonantHy
                    | Phoneme::ConsonantJ
                    | Phoneme::ConsonantK
                    | Phoneme::ConsonantKw
                    | Phoneme::ConsonantKy
                    | Phoneme::ConsonantM
                    | Phoneme::ConsonantMy
                    | Phoneme::ConsonantN
                    | Phoneme::ConsonantNy
                    | Phoneme::ConsonantP
                    | Phoneme::ConsonantPy
                    | Phoneme::ConsonantR
                    | Phoneme::ConsonantRy
                    | Phoneme::ConsonantS
                    | Phoneme::ConsonantSh
                    | Phoneme::ConsonantT
                    | Phoneme::ConsonantTs
                    | Phoneme::ConsonantTy
                    | Phoneme::ConsonantV
                    | Phoneme::ConsonantW
                    | Phoneme::ConsonantY
                    | Phoneme::ConsonantZ => Err(error(InvalidQueryErrorSource::IsConsonant)),
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
            MorableCl,
            VoicedVowelE,
            VoicedVowelI,
            VoicedVowelO,
            VoicedVowelU,
        )
    }
}

impl FromStr for NonConsonant {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_with_inner_error(s).map_err(Into::into)
    }
}

impl Sil {
    pub fn get(&self) -> &str {
        self.as_ref()
    }
}

impl Default for Sil {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl FromStr for Sil {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or_else(|| {
            ErrorRepr::InvalidQuery(InvalidQueryError {
                what: "sil音素",
                value: Some(Box::new(s.to_owned())),
                source: Some(InvalidQueryErrorSource::MustContainSil),
            })
            .into()
        })
    }
}

impl<'de> Deserialize<'de> for Sil {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        return deserializer.deserialize_str(Visitor);

        struct Visitor;

        impl de::Visitor<'_> for Visitor {
            type Value = Sil;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(fmt, "a string that contains \"sil\"")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                s.parse()
                    .map_err(|_| de::Error::invalid_value(Unexpected::Str(s), &self))
            }
        }
    }
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

impl From<PhonemeCode> for Phoneme {
    fn from(phoneme: PhonemeCode) -> Self {
        macro_rules! convert {
            ($($variant:ident),* $(,)?) => {
                match phoneme {
                    $(PhonemeCode::$variant => Self::$variant,)*
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

impl From<Consonant> for PhonemeCode {
    fn from(consonant: Consonant) -> Self {
        use PhonemeCode::*;

        macro_rules! convert {
            ($($variant:tt),* $(,)?) => {
                match consonant {
                    $(Consonant::$variant => paste!([<Consonant $variant>])),*
                }
            };
        }

        convert!(
            B, By, Ch, D, Dy, F, G, Gw, Gy, H, Hy, J, K, Kw, Ky, M, My, N, Ny, P, Py, R, Ry, S, Sh,
            T, Ts, Ty, V, W, Y, Z,
        )
    }
}

impl From<NonConsonant> for PhonemeCode {
    fn from(non_consonant: NonConsonant) -> Self {
        macro_rules! convert {
            ($($variant:ident),* $(,)?) => {
                match non_consonant {
                    $(NonConsonant::$variant => Self::$variant,)*
                    NonConsonant::Sil(_) => Self::space_phoneme(),
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
            MorableCl,
            VoicedVowelE,
            VoicedVowelI,
            VoicedVowelO,
            VoicedVowelU,
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

impl From<OptionalConsonant> for &'static str {
    fn from(phoneme: OptionalConsonant) -> Self {
        macro_rules! convert {
            ($($s:tt),* $(,)?) => {
                match phoneme {
                    $(optional_consonant!($s) => $s),*
                }
            };
        }

        convert!(
            "", "b", "by", "ch", "d", "dy", "f", "g", "gw", "gy", "h", "hy", "j", "k", "kw", "ky",
            "m", "my", "n", "ny", "p", "py", "r", "ry", "s", "sh", "t", "ts", "ty", "v", "w", "y",
            "z"
        )
    }
}

impl From<OptionalConsonant> for Option<Consonant> {
    fn from(consonant: OptionalConsonant) -> Self {
        use OptionalConsonant::*;

        macro_rules! convert {
            ($($variant:tt),* $(,)?) => {
                match consonant {
                    None => Option::None,
                    $(paste!([<Consonant $variant>]) => Some(Consonant::$variant),)*
                }
            };
        }

        convert!(
            B, By, Ch, D, Dy, F, G, Gw, Gy, H, Hy, J, K, Kw, Ky, M, My, N, Ny, P, Py, R, Ry, S, Sh,
            T, Ts, Ty, V, W, Y, Z,
        )
    }
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

impl From<MoraTail> for NonConsonant {
    fn from(phoneme: MoraTail) -> Self {
        macro_rules! convert {
            ($($variant:ident),* $(,)?) => {
                match phoneme {
                    $(MoraTail::$variant => Self::$variant,)*
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
            MorableCl,
            VoicedVowelE,
            VoicedVowelI,
            VoicedVowelO,
            VoicedVowelU,
        )
    }
}

impl NonPauBaseVowel {
    pub(in super::super) fn to_unvoiced(self) -> Option<MoraTail> {
        match self {
            Self::VoicedVowelA => Some(MoraTail::UnvoicedVowelA),
            Self::VoicedVowelI => Some(MoraTail::UnvoicedVowelI),
            Self::VoicedVowelU => Some(MoraTail::UnvoicedVowelU),
            Self::VoicedVowelE => Some(MoraTail::UnvoicedVowelE),
            Self::VoicedVowelO => Some(MoraTail::UnvoicedVowelO),
            _ => None,
        }
    }
}

impl From<NonPauBaseVowel> for &'static str {
    fn from(phoneme: NonPauBaseVowel) -> Self {
        macro_rules! convert {
            ($($s:tt),* $(,)?) => {
                match phoneme {
                    $(non_pau_base_vowel!($s) => $s),*
                }
            };
        }

        convert!("N", "a", "cl", "e", "i", "o", "u")
    }
}

impl From<NonPauBaseVowel> for MoraTail {
    fn from(phoneme: NonPauBaseVowel) -> Self {
        bytemuck::checked::cast(phoneme)
    }
}

impl From<NonPauBaseVowel> for NonConsonant {
    fn from(phoneme: NonPauBaseVowel) -> Self {
        MoraTail::from(phoneme).into()
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

impl From<OptionalConsonant> for Option<PhonemeCode> {
    fn from(consonant: OptionalConsonant) -> Self {
        bytemuck::checked::try_cast(consonant)
            .inspect_err(|&err| {
                assert_eq!(
                    CheckedCastError::InvalidBitPattern,
                    err,
                    "there should be no size/alignment issues",
                );
            })
            .ok()
    }
}

impl From<NonPauBaseVowel> for PhonemeCode {
    fn from(phoneme: NonPauBaseVowel) -> Self {
        bytemuck::checked::cast(phoneme)
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator as _;

    use super::super::{MoraTail, NonPauBaseVowel};

    #[test]
    fn non_pau_base_vowel_into_mora_tail_works() {
        for vowel in NonPauBaseVowel::iter() {
            let _ = MoraTail::from(vowel);
        }
    }
}
