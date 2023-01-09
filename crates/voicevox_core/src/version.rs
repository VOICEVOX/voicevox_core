pub const fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    #[rstest]
    fn get_version_works() {
        assert_eq!("0.0.0", get_version());
    }
}
