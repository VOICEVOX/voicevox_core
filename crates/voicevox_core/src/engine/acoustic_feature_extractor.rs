use std::{collections::HashMap, sync::LazyLock};

use derive_getters::Getters;

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

#[derive(Debug, Clone, PartialEq, Default, Getters)]
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

    pub(super) fn new(phoneme: &str) -> Self {
        let phoneme = if phoneme.contains("sil") {
            Self::space_phoneme()
        } else {
            if !PHONEME_MAP.contains_key(phoneme) {
                panic!("invalid phoneme: {phoneme:?}");
            }
            phoneme.to_owned()
        };
        Self { phoneme }
    }

    pub(crate) fn phoneme_id(&self) -> i64 {
        if self.phoneme.is_empty() {
            -1
        } else {
            *PHONEME_MAP.get(&self.phoneme.as_str()).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::OjtPhoneme;

    const STR_HELLO_HIHO: &str = "sil k o N n i ch i w a pau h i h o d e s U sil";

    fn ojt_hello_hiho() -> Vec<OjtPhoneme> {
        STR_HELLO_HIHO
            .split_whitespace()
            .map(OjtPhoneme::new)
            .collect()
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
