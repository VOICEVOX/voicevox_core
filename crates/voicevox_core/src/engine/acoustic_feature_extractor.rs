use derive_getters::Getters;
use derive_new::new;
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[rustfmt::skip]
const PHONEME_LIST: [&str; 45] = [
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

static PHONEME_MAP: Lazy<HashMap<&str, i64>> = Lazy::new(|| {
    let mut m = HashMap::new();
    for (i, s) in PHONEME_LIST.iter().enumerate() {
        m.insert(*s, i as i64);
    }
    m
});

#[derive(Debug, Clone, PartialEq, new, Default, Getters)]
pub struct OjtPhoneme {
    phoneme: String,
    #[allow(dead_code)]
    start: f32,
    #[allow(dead_code)]
    end: f32,
}

impl OjtPhoneme {
    pub(crate) const NUM_PHONEME: usize = PHONEME_LIST.len();

    pub fn space_phoneme() -> String {
        "pau".into()
    }

    pub fn phoneme_id(&self) -> i64 {
        if self.phoneme.is_empty() {
            -1
        } else {
            *PHONEME_MAP.get(&self.phoneme.as_str()).unwrap()
        }
    }

    pub fn convert(phonemes: &[OjtPhoneme]) -> Vec<OjtPhoneme> {
        let mut phonemes = phonemes.to_owned();
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
    use super::*;
    use pretty_assertions::assert_eq;

    use crate::*;

    const STR_HELLO_HIHO: &str = "sil k o N n i ch i w a pau h i h o d e s U sil";

    fn base_hello_hiho() -> Vec<OjtPhoneme> {
        STR_HELLO_HIHO
            .split_whitespace()
            .enumerate()
            .map(|(i, s)| OjtPhoneme::new(s.into(), i as f32, (i + 1) as f32))
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
        assert_eq!(PHONEME_LIST[index], phoneme_str);
    }

    #[rstest]
    fn test_phoneme_map_has_enough_elements() {
        assert_eq!(OjtPhoneme::NUM_PHONEME, PHONEME_MAP.len());
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
    #[case(ojt_hello_hiho(), 9, OjtPhoneme::new("a".into(), 9., 10.), true)]
    #[case(ojt_hello_hiho(), 9, OjtPhoneme::new("k".into(), 9., 10.), false)]
    #[case(ojt_hello_hiho(), 9, OjtPhoneme::new("a".into(), 10., 11.), false)]
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
