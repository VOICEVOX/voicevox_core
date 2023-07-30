use anyhow::Result;
use jni::JNIEnv;
pub static PACKAGE_NAME: &str = "jp/Hiroshiba/VoicevoxCore";

#[macro_export]
macro_rules! object {
    ($name: literal) => {
        concat!("Ljp/Hiroshiba/VoicevoxCore/", $name, ";")
    };
}

pub fn throw_if_err<T, F>(mut env: JNIEnv, fallback: T, inner: F) -> T
where
    F: FnOnce(&mut JNIEnv) -> Result<T>,
{
    match inner(&mut env) {
        Ok(value) => value as _,
        Err(error) => {
            env.throw_new("jp/Hiroshiba/VoicevoxCore/VoicevoxError", error.to_string())
                .unwrap();
            fallback
        }
    }
}
