use anyhow::Result;
use jni::JNIEnv;

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
            // Java側の例外は無視する。
            // env.exception_clear()してもいいが、errorのメッセージは"Java exception was thrown"
            // となり、デバッグが困難になるため、そのままにしておく。
            if !env.exception_check().unwrap_or(false) {
                env.throw_new(
                    "jp/Hiroshiba/VoicevoxCore/VoicevoxException",
                    error.to_string(),
                )
                .unwrap_or_else(|_| panic!("Failed to throw exception, original error: {}", error));
            }
            fallback
        }
    }
}
