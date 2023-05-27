include!(concat!(env!("OUT_DIR"), "/version_macro.rs"));

#[cfg(test)]
mod tests {
    use crate::*;
    #[rstest]
    fn get_version_works() {
        assert_eq!("0.0.0", version!());
    }
}
