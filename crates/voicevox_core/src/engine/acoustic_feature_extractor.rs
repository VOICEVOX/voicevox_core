use std::{collections::HashMap, sync::LazyLock};

use derive_getters::Getters;
use derive_new::new;
use enum_map::Enum;
use strum::EnumString;

#[derive(Clone, Copy, Enum, EnumString)]
pub(crate) enum Phoneme {
    #[strum(serialize = "pau")]
    ShortPause,

    #[strum(serialize = "A")]
    UnvoicedA,

    #[strum(serialize = "E")]
    UnvoicedE,

    #[strum(serialize = "I")]
    UnvoicedI,

    #[strum(serialize = "N")]
    Hatsuon,

    #[strum(serialize = "O")]
    UnvoicedO,

    #[strum(serialize = "U")]
    UnvoicedU,

    #[strum(serialize = "a")]
    VoicedA,

    #[strum(serialize = "b")]
    どうしよう,

    #[strum(serialize = "by")]
    どうしよう,

    #[strum(serialize = "ch")]
    どうしよう,

    #[strum(serialize = "cl")]
    どうしよう,

    #[strum(serialize = "d")]
    どうしよう,

    #[strum(serialize = "dy")]
    どうしよう,

    #[strum(serialize = "e")]
    どうしよう,

    #[strum(serialize = "f")]
    どうしよう,

    #[strum(serialize = "g")]
    どうしよう,

    #[strum(serialize = "gw")]
    どうしよう,

    #[strum(serialize = "gy")]
    どうしよう,

    #[strum(serialize = "h")]
    どうしよう,

    #[strum(serialize = "hy")]
    どうしよう,

    #[strum(serialize = "i")]
    どうしよう,

    #[strum(serialize = "j")]
    どうしよう,

    #[strum(serialize = "k")]
    どうしよう,

    #[strum(serialize = "kw")]
    どうしよう,

    #[strum(serialize = "ky")]
    どうしよう,

    #[strum(serialize = "m")]
    どうしよう,

    #[strum(serialize = "my")]
    どうしよう,

    #[strum(serialize = "n")]
    どうしよう,

    #[strum(serialize = "ny")]
    どうしよう,

    #[strum(serialize = "o")]
    どうしよう,

    #[strum(serialize = "p")]
    どうしよう,

    #[strum(serialize = "py")]
    どうしよう,

    #[strum(serialize = "r")]
    どうしよう,

    #[strum(serialize = "ry")]
    どうしよう,

    #[strum(serialize = "s")]
    どうしよう,

    #[strum(serialize = "sh")]
    どうしよう,

    #[strum(serialize = "t")]
    どうしよう,

    #[strum(serialize = "ts")]
    どうしよう,

    #[strum(serialize = "ty")]
    どうしよう,

    #[strum(serialize = "u")]
    どうしよう,

    #[strum(serialize = "v")]
    どうしよう,

    #[strum(serialize = "w")]
    どうしよう,

    #[strum(serialize = "y")]
    どうしよう,

    #[strum(serialize = "z")]
    どうしよう,
}

#[rustfmt::skip]
const PHONEME_LIST: &[&str] = &[
    "pau",
    "A",
    "E",
    "I",
    "N",
    "O",
    "U",
    "a",
    "b",
    "by",
    "ch",
    "cl",
    "d",
    "dy",
    "e",
    "f",
    "g",
    "gw",
    "gy",
    "h",
    "hy",
    "i",
    "j",
    "k",
    "kw",
    "ky",
    "m",
    "my",
    "n",
    "ny",
    "o",
    "p",
    "py",
    "r",
    "ry",
    "s",
    "sh",
    "t",
    "ts",
    "ty",
    "u",
    "v",
    "w",
    "y",
    "z",
];

static PHONEME_MAP: LazyLock<HashMap<&str, i64>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for (i, s) in PHONEME_LIST.iter().enumerate() {
        m.insert(*s, i as i64);
    }
    m
});

#[derive(Debug, Clone, PartialEq, new, Default, Getters)]
pub(crate) struct OjtPhoneme {
    phoneme: String,
}

impl OjtPhoneme {
    pub(crate) const fn num_phoneme() -> usize {
        PHONEME_LIST.len() // == PHONEME_MAP.len()
    }

    fn space_phoneme() -> String {
        "pau".into()
    }

    pub(crate) fn phoneme_id(&self) -> i64 {
        if self.phoneme.is_empty() {
            -1
        } else {
            *PHONEME_MAP.get(&self.phoneme.as_str()).unwrap()
        }
    }

    pub(super) fn convert(phonemes: &[OjtPhoneme]) -> Vec<OjtPhoneme> {
        let mut phonemes = phonemes.to_owned();
        // TODO: Rust 2024にしたらlet chainに戻す
        #[cfg(any())]
        __! {
        if let Some(first_phoneme) = phonemes.first_mut()
            && first_phoneme.phoneme.contains("sil")
        {
            first_phoneme.phoneme = OjtPhoneme::space_phoneme();
        }
        if let Some(last_phoneme) = phonemes.last_mut()
            && last_phoneme.phoneme.contains("sil")
        {
            last_phoneme.phoneme = OjtPhoneme::space_phoneme();
        }
        }
        if let Some(first_phoneme) = phonemes.first_mut() {
            if first_phoneme.phoneme.contains("sil") {
                first_phoneme.phoneme = OjtPhoneme::space_phoneme();
            }
        }
        if let Some(last_phoneme) = phonemes.last_mut() {
            if last_phoneme.phoneme.contains("sil") {
                last_phoneme.phoneme = OjtPhoneme::space_phoneme();
            }
        }
        phonemes
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::OjtPhoneme;

    const STR_HELLO_HIHO: &str = "sil k o N n i ch i w a pau h i h o d e s U sil";

    fn base_hello_hiho() -> Vec<OjtPhoneme> {
        STR_HELLO_HIHO
            .split_whitespace()
            .map(ToOwned::to_owned)
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
        assert_eq!(super::PHONEME_LIST[index], phoneme_str);
    }

    #[rstest]
    fn test_num_phoneme_works() {
        assert_eq!(OjtPhoneme::num_phoneme(), 45);
    }

    #[rstest]
    fn test_space_phoneme_works() {
        assert_eq!(OjtPhoneme::space_phoneme(), "pau");
    }

    #[rstest]
    #[case(ojt_hello_hiho(), "pau k o N n i ch i w a pau h i h o d e s U pau")]
    fn test_convert_works(#[case] ojt_phonemes: Vec<OjtPhoneme>, #[case] expected: &str) {
        let ojt_str_hello_hiho: String = ojt_phonemes
            .iter()
            .map(|phoneme| phoneme.phoneme().clone())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(ojt_str_hello_hiho, expected);
    }

    #[rstest]
    #[case(ojt_hello_hiho(), 9, OjtPhoneme::new("a".into()), true)]
    #[case(ojt_hello_hiho(), 9, OjtPhoneme::new("k".into()), false)]
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
