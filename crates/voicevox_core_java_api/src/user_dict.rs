use jni::objects::JClass;
use std::{borrow::Cow, sync::Arc};

use crate::common::{throw_if_err, JavaApiError, RUNTIME};
use jni::{
    objects::{JObject, JString},
    sys::jobject,
    JNIEnv,
};

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDict_rsNew<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = voicevox_core::tokio::UserDict::new();

        env.set_rust_field(&this, "handle", Arc::new(internal))?;

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDict_rsAddWord<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    word_json: JString<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::tokio::UserDict>>(&this, "handle")?
            .clone();

        let word_json = env.get_string(&word_json)?;
        let word_json = &Cow::from(&word_json);

        let word: voicevox_core::UserDictWord =
            serde_json::from_str(word_json).map_err(JavaApiError::DeJson)?;

        let uuid = internal.add_word(word)?.hyphenated().to_string();
        let uuid = env.new_string(uuid)?;

        Ok(uuid.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDict_rsUpdateWord<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    uuid: JString<'local>,
    word_json: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::tokio::UserDict>>(&this, "handle")?
            .clone();

        let uuid = env.get_string(&uuid)?;
        let uuid = Cow::from(&uuid).parse()?;
        let word_json = env.get_string(&word_json)?;
        let word_json = &Cow::from(&word_json);

        let word: voicevox_core::UserDictWord =
            serde_json::from_str(word_json).map_err(JavaApiError::DeJson)?;

        internal.update_word(uuid, word)?;

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDict_rsRemoveWord<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    uuid: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::tokio::UserDict>>(&this, "handle")?
            .clone();

        let uuid = env.get_string(&uuid)?;
        let uuid = Cow::from(&uuid).parse()?;

        internal.remove_word(uuid)?;

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDict_rsImportDict<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    other_dict: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::tokio::UserDict>>(&this, "handle")?
            .clone();
        let other_dict = env
            .get_rust_field::<_, _, Arc<voicevox_core::tokio::UserDict>>(&other_dict, "handle")?
            .clone();

        internal.import(&other_dict)?;

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDict_rsLoad<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    path: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::tokio::UserDict>>(&this, "handle")?
            .clone();

        let path = env.get_string(&path)?;
        let path = &Cow::from(&path);

        RUNTIME.block_on(internal.load(path))?;

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDict_rsSave<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    path: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::tokio::UserDict>>(&this, "handle")?
            .clone();

        let path = env.get_string(&path)?;
        let path = &Cow::from(&path);

        RUNTIME.block_on(internal.save(path))?;

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDict_rsGetWords<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::tokio::UserDict>>(&this, "handle")?
            .clone();

        let words = internal.to_json();
        let words = env.new_string(words)?;

        Ok(words.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDict_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        env.take_rust_field(&this, "handle")?;
        Ok(())
    })
}

#[no_mangle]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDict_rsToZenkaku<'local>(
    env: JNIEnv<'local>,
    _cls: JClass<'local>,
    text: JString<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let text = env.get_string(&text)?;
        let text = &Cow::from(&text);

        let text = voicevox_core::__internal::to_zenkaku(text);

        let text = env.new_string(text)?;
        Ok(text.into_raw())
    })
}

#[no_mangle]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDict_rsValidatePronunciation<'local>(
    env: JNIEnv<'local>,
    _cls: JClass<'local>,
    text: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let text = env.get_string(&text)?;
        let text = &Cow::from(&text);

        voicevox_core::__internal::validate_pronunciation(text)?;

        Ok(())
    })
}
