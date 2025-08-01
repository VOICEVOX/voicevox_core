/// 本クレートの`package.version`。
#[cfg_attr(doc, doc(alias = "voicevox_get_version"))]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[rstest]
    fn get_version_works() {
        assert_eq!("0.0.0", super::VERSION);
    }
}
