use std::{borrow::Cow, ptr};

use crate::common::{JavaApiError, throw_if_err};
use jni::{
    JNIEnv,
    objects::{JClass, JObject, JString, JValueGen},
    sys::jstring,
};
use voicevox_core::{AccentPhrase, AudioQuery, Mora};

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
    throw_if_err(env, (), |env| {
        let gson = env.new_object("com/google/gson/Gson", "()V", &[])?;

        let audio_query = &env
            .call_method(
                gson,
                "toJson",
                "(Ljava/lang/Object;)Ljava/lang/String;",
                &[JValueGen::Object(&this)],
            )?
            .l()?
            .into();
        let audio_query = &env.get_string(audio_query)?;
        let audio_query = &Cow::from(audio_query);

        serde_json::from_str::<AudioQuery>(audio_query)
            .map_err(JavaApiError::DeJson)?
            .validate()?;
        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_AccentPhrase_rsValidate(
    env: JNIEnv<'_>,
    this: JObject<'_>,
) {
    throw_if_err(env, (), |env| {
        let gson = env.new_object("com/google/gson/Gson", "()V", &[])?;

        let audio_query = &env
            .call_method(
                gson,
                "toJson",
                "(Ljava/lang/Object;)Ljava/lang/String;",
                &[JValueGen::Object(&this)],
            )?
            .l()?
            .into();
        let audio_query = &env.get_string(audio_query)?;
        let audio_query = &Cow::from(audio_query);

        serde_json::from_str::<AccentPhrase>(audio_query)
            .map_err(JavaApiError::DeJson)?
            .validate()?;
        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_Mora_rsValidate(
    env: JNIEnv<'_>,
    this: JObject<'_>,
) {
    throw_if_err(env, (), |env| {
        let gson = env.new_object("com/google/gson/Gson", "()V", &[])?;

        let audio_query = &env
            .call_method(
                gson,
                "toJson",
                "(Ljava/lang/Object;)Ljava/lang/String;",
                &[JValueGen::Object(&this)],
            )?
            .l()?
            .into();
        let audio_query = &env.get_string(audio_query)?;
        let audio_query = &Cow::from(audio_query);

        serde_json::from_str::<Mora>(audio_query)
            .map_err(JavaApiError::DeJson)?
            .validate()?;
        Ok(())
    })
}
