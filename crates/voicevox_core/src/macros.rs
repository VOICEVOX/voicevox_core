#[cfg(test)]
pub(crate) mod tests {
    macro_rules! assert_result {
        ($($arg:tt)*) => {
            |__r| ::assert_matches::assert_matches!(__r, $($arg)*)
        };
    }
    pub(crate) use assert_result;
}
