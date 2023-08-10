/// 本クレートの`package.version`。
///
/// C APIやPython API側からこの値が使われるべきではない。
/// 現在はまだRust APIを外部提供していないため、この定数はどこからも参照されていないはずである。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    #[rstest]
    fn get_version_works() {
        assert_eq!("0.0.0", VERSION);
    }
}