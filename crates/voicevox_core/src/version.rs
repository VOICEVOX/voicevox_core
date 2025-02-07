/// 本クレートの`package.version`。
///
/// TODO: ↓ 「Rust APIを外部提供」はもう嘘
///
/// C APIやPython API側からこの値が使われるべきではない。現在はまだRust APIを外部提供していないため、この定数はどこからも参照されていないはずである。
#[doc(alias = "voicevox_get_version")]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use rstest::rstest;

    #[rstest]
    fn get_version_works() {
        assert_eq!("0.0.0", super::VERSION);
    }
}
