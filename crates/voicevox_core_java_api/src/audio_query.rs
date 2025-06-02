use std::ptr;

use crate::common::{JavaApiError, throw_if_err};
use jni::{
    JNIEnv,
    objects::{JClass, JString},
    sys::jstring,
};
use voicevox_core::AudioQuery;

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_AudioQuery_rsFromAccentPhrases(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
    accent_phrases: JString<'_>,
) -> jstring {
    throw_if_err(env, ptr::null_mut(), |env| {
        let accent_phrases = &String::from(env.get_string(&accent_phrases)?);
        let accent_phrases = serde_json::from_str(accent_phrases).map_err(JavaApiError::DeJson)?;
        let query = &AudioQuery::from_accent_phrases(accent_phrases);
        let query = serde_json::to_string(query).expect("should not fail");
        let query = env.new_string(query)?;
        Ok(query.into_raw())
    })
}
