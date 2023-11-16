use super::DecryptModelError;

pub(super) fn decrypt(content: &[u8]) -> std::result::Result<Vec<u8>, DecryptModelError> {
    Ok(content.to_owned())
}
