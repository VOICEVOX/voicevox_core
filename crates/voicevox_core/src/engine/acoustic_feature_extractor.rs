use derive_getters::Getters;
use derive_new::new;
use once_cell::sync::Lazy;
use std::collections::HashMap;

macro_rules! make_phoneme_map {
    ($(($key:expr, $value:expr)),*) => {
        {
            let mut m = HashMap::new();
            $(m.insert($key.to_string(), $value);)*
            m
        }
    };
}

#[rustfmt::skip]
static PHONEME_MAP: Lazy<HashMap<String, i64>> = Lazy::new(|| {
    make_phoneme_map!(
        ("pau", 0), ("A", 1),   ("E", 2),   ("I", 3),   ("N", 4),   ("O", 5),   ("U", 6),   ("a", 7),   ("b", 8),
        ("by", 9),  ("ch", 10), ("cl", 11), ("d", 12),  ("dy", 13), ("e", 14),  ("f", 15),  ("g", 16),  ("gw", 17),
        ("gy", 18), ("h", 19),  ("hy", 20), ("i", 21),  ("j", 22),  ("k", 23),  ("kw", 24), ("ky", 25), ("m", 26),
        ("my", 27), ("n", 28),  ("ny", 29), ("o", 30),  ("p", 31),  ("py", 32), ("r", 33),  ("ry", 34), ("s", 35),
        ("sh", 36), ("t", 37),  ("ts", 38), ("ty", 39), ("u", 40),  ("v", 41),  ("w", 42),  ("y", 43),  ("z", 44)
    )
});

#[derive(Debug, Clone, new, Default, Getters)]
pub struct OjtPhoneme {
    phoneme: String,
    #[allow(dead_code)]
    start: f32,
    #[allow(dead_code)]
    end: f32,
}

impl OjtPhoneme {
    #[allow(dead_code)]
    pub fn num_phoneme() -> usize {
        PHONEME_MAP.len()
    }

    pub fn space_phoneme() -> String {
        "pau".into()
    }

    pub fn phoneme_id(&self) -> i64 {
        if self.phoneme.is_empty() {
            -1
        } else {
            *PHONEME_MAP.get(&self.phoneme).unwrap()
        }
    }

    pub fn convert(phonemes: &[OjtPhoneme]) -> Vec<OjtPhoneme> {
        let mut phonemes = phonemes.to_owned();
        if let Some(first_phoneme) = phonemes.first_mut() {
            if !first_phoneme.phoneme.contains("sil") {
                first_phoneme.phoneme = OjtPhoneme::space_phoneme();
            }
        }
        if let Some(last_phoneme) = phonemes.last_mut() {
            if !last_phoneme.phoneme.contains("sil") {
                last_phoneme.phoneme = OjtPhoneme::space_phoneme();
            }
        }
        phonemes
    }
}
