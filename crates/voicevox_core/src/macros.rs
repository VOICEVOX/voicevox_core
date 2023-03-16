#[cfg(test)]
pub(crate) mod tests {
    use std::fmt::{self, Debug};

    use pretty_assertions::StrComparison;

    /// 2つの`"{:#?}"`が等しいかを、pretty\_assertions風にassertする。
    ///
    /// `io::Error`や`anyhow::Error`を抱えていて`PartialEq`実装が難しい型に使う。
    ///
    /// # Panics
    ///
    /// 2つの`"{:#?}"`が等しくないとき、assertの失敗としてパニックする。
    macro_rules! assert_debug_fmt_eq {
        ($left:expr, $right:expr $(,)?) => {{
            crate::macros::tests::__assert_debug_fmt(&$left, &$right, None)
        }};
        ($left:expr, $right:expr, $($arg:tt)*) => {{
            crate::macros::tests::__assert_debug_fmt(&$left, &$right, Some(format_args!($($arg)*)))
        }};
    }
    pub(crate) use assert_debug_fmt_eq;

    #[track_caller]
    pub(crate) fn __assert_debug_fmt<T: Debug>(
        left: &T,
        right: &T,
        args: Option<fmt::Arguments<'_>>,
    ) {
        let (left, right) = (format!("{left:#?}"), format!("{right:#?}"));
        if left != right {
            panic!(
                r#"assertion failed: `("{{left:#?}}" == "{{right:#?}}")`{}

{}
"#,
                match args {
                    Some(args) => format!(": {args}"),
                    None => "".to_owned(),
                },
                StrComparison::new(&left, &right),
            );
        }
    }
}
