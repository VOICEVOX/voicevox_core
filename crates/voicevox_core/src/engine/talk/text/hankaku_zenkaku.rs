use std::sync::LazyLock;

use regex::Regex;

/// 一部の種類の文字を、全角文字に置き換える。
///
/// 具体的には
///
/// - "!"から"~"までの範囲の文字(数字やアルファベット)は、対応する全角文字に
/// - " "などの目に見えない文字は、まとめて全角スペース(0x3000)に
///
/// 変換する。
pub(crate) fn to_zenkaku(s: &str) -> String {
    // 元実装：https://github.com/VOICEVOX/voicevox/blob/69898f5dd001d28d4de355a25766acb0e0833ec2/src/components/DictionaryManageDialog.vue#L379-L387

    static SPACE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\p{Z}").unwrap());

    SPACE_REGEX
        .replace_all(s, "\u{3000}")
        .chars()
        .map(|c| match u32::from(c) {
            i @ 0x21..=0x7e => char::from_u32(0xfee0 + i).unwrap_or(c),
            _ => c,
        })
        .collect()
}

pub(crate) fn to_hankaku(s: &str) -> String {
    s.chars()
        .map(|c| match u32::from(c) {
            c @ 0xFF01..=0xFF5E => char::from_u32(c - 0xFF01 + 0x21).unwrap(),
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[rstest]
    #[case("abcdefg", "ａｂｃｄｅｆｇ")]
    #[case("あいうえお", "あいうえお")]
    #[case("a_b_c_d_e_f_g", "ａ＿ｂ＿ｃ＿ｄ＿ｅ＿ｆ＿ｇ")]
    #[case("a b c d e f g", "ａ　ｂ　ｃ　ｄ　ｅ　ｆ　ｇ")]
    fn to_zenkaku_works(#[case] before: &str, #[case] after: &str) {
        assert_eq!(super::to_zenkaku(before), after);
    }
}
