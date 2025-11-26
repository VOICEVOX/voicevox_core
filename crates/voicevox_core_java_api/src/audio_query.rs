use std::{borrow::Cow, ptr};

use crate::common::{JavaApiError, JavaApiResult, throw_if_err};
use easy_ext::ext;
use jni::{
    JNIEnv,
    objects::{JClass, JObject, JString, JValueGen},
    sys::jstring,
};
use voicevox_core::{__internal::interop::Validate, AccentPhrase, AudioQuery, Mora};

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

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_AudioQuery_rsValidate(
    env: JNIEnv<'_>,
    this: JObject<'_>,
) {
    throw_if_err(env, (), |env| AudioQuery::validate_json(env, this))
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_AccentPhrase_rsValidate(
    env: JNIEnv<'_>,
    this: JObject<'_>,
) {
    throw_if_err(env, (), |env| AccentPhrase::validate_json(env, this))
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_Mora_rsValidate(
    env: JNIEnv<'_>,
    this: JObject<'_>,
) {
    throw_if_err(env, (), |env| Mora::validate_json(env, this))
}

#[ext]
impl<T: Validate> T {
    fn validate_json(env: &mut JNIEnv<'_>, this: JObject<'_>) -> JavaApiResult<()> {
        let gson = env.new_object("com/google/gson/Gson", "()V", &[])?;

        let this = &env
            .call_method(
                gson,
                "toJson",
                "(Ljava/lang/Object;)Ljava/lang/String;",
                &[JValueGen::Object(&this)],
            )?
            .l()?
            .into();
        let this = &env.get_string(this)?;
        let this = &Cow::from(this);

        serde_json::from_str::<Self>(this)
            .map_err(JavaApiError::DeJson)?
            .validate()?;
        Ok(())
    }
}
