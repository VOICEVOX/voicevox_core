//! 英単語をカタカナ読みにする処理。

pub(super) use self::hankaku_alphabets::HankakuAlphabets;

impl HankakuAlphabets {
    pub(super) fn convert_english_to_katakana(&self) -> String {
        self.split_into_words()
            .map(|word| {
                if word.should_convert_english_to_katakana() {
                    kanalizer::convert(&word.get().to_ascii_lowercase())
                        .perform()
                        .unwrap_or_else(|e| todo!("{e}"))
                } else {
                    word.convert_as_char_wise_katakana()
                }
            })
            .collect()
    }

    fn should_convert_english_to_katakana(&self) -> bool {
        if self.get().chars().count() == 1 {
            return false;
        }
        if self.get() == self.get().to_uppercase() {
            return false;
        }
        true
    }
}

mod hankaku_alphabets {
    use std::sync::LazyLock;

    use ref_cast::{ref_cast_custom, RefCastCustom};
    use regex::Regex;

    #[derive(RefCastCustom)]
    #[repr(transparent)]
    pub(in super::super) struct HankakuAlphabets(
        str, // Invariant: all characters must be ASCII alphabets.
    );

    impl HankakuAlphabets {
        pub(in super::super) fn new(s: &str) -> Option<&Self> {
            return is_hankaku_alphabets(s).then(|| Self::new_(s));

            fn is_hankaku_alphabets(text: &str) -> bool {
                text.chars().all(|c| c.is_ascii_alphabetic())
            }
        }

        #[ref_cast_custom]
        const fn new_(s: &str) -> &Self;

        pub(in super::super) fn get(&self) -> &str {
            &self.0
        }

        pub(super) fn split_into_words(&self) -> impl Iterator<Item = &Self> {
            static REGEX: LazyLock<Regex> = LazyLock::new(|| "[a-zA-Z][a-z]*".parse().unwrap());

            REGEX.find_iter(&self.0).map(|m| Self::new_(m.as_str()))
        }

        pub(super) fn convert_as_char_wise_katakana(&self) -> String {
            self.0
                .chars()
                .map(|c| match c {
                    'A' | 'a' => "エー",
                    'B' | 'b' => "ビー",
                    'C' | 'c' => "シー",
                    'D' | 'd' => "ディー",
                    'E' | 'e' => "イー",
                    'F' | 'f' => "エフ",
                    'G' | 'g' => "ジー",
                    'H' | 'h' => "エイチ",
                    'I' | 'i' => "アイ",
                    'J' | 'j' => "ジェー",
                    'K' | 'k' => "ケー",
                    'L' | 'l' => "エル",
                    'M' | 'm' => "エム",
                    'N' | 'n' => "エヌ",
                    'O' | 'o' => "オー",
                    'P' | 'p' => "ピー",
                    'Q' | 'q' => "キュー",
                    'R' | 'r' => "アール",
                    'S' | 's' => "エス",
                    'T' | 't' => "ティー",
                    'U' | 'u' => "ユー",
                    'V' | 'v' => "ブイ",
                    'W' | 'w' => "ダブリュー",
                    'X' | 'x' => "エックス",
                    'Y' | 'y' => "ワイ",
                    'Z' | 'z' => "ズィー",
                    _ => unreachable!("the invariant must be held"),
                })
                .collect()
        }
    }
}
