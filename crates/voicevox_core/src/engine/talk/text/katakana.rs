use std::sync::LazyLock;

use regex::Regex;

pub(in super::super) fn count_moras(pron: &str) -> usize {
    // 元実装：https://github.com/VOICEVOX/voicevox_engine/blob/39747666aa0895699e188f3fd03a0f448c9cf746/voicevox_engine/model.py#L219-L228

    static MORA_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        // TODO: Rust 2024にしたら`any()`を`false`に
        #[cfg_attr(any(), rustfmt::skip)] // `rule_one_mora`の行が100文字を超過してしまうため
        Regex::new(concat!(
            "(?:",
            "[イ][ェ]|[ヴ][ャュョ]|[トド][ゥ]|[テデ][ィャュョ]|[デ][ェ]|[クグ][ヮ]|", // rule_others
            "[キシチニヒミリギジビピ][ェャュョ]|",                                    // rule_line_i
            "[ツフヴ][ァ]|[ウスツフヴズ][ィ]|[ウツフヴ][ェォ]|",                      // rule_line_u
            "[ァ-ヴー]",                                                              // rule_one_mora
            ")",
        ))
        .unwrap()
    });

    MORA_REGEX.find_iter(pron).count()
}

// TODO: ENGINEのテストを持ってくる。
