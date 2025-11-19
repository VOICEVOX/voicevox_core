use jni::objects::JClass;
use std::{borrow::Cow, sync::Arc};

use crate::common::{JNIEnvExt as _, JavaApiResult, throw_if_err};
use jni::{
    JNIEnv,
    objects::{JObject, JString},
    sys::jobject,
};
use serde_json::json;

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_UserDict_rsNew<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = voicevox_core::blocking::UserDict::new();

        // SAFETY:
        // - The safety contract must be upheld by the caller.
        // - `jp.hiroshiba.voicevoxcore.blocking.UserDict.handle` must correspond to
        //   `Arc<voicevox_core::blocking::UserDict>`.
        unsafe { env.set_rust_field(&this, "handle", Arc::new(internal)) }?;

        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_UserDict_rsAddWord<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    word: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.UserDict.handle` must correspond to
            //   `Arc<voicevox_core::blocking::UserDict>`.
            env.get_rust_field::<_, _, Arc<voicevox_core::blocking::UserDict>>(&this, "handle")
        }?
        .clone();

        let uuid = internal.add_word(word_from_java(env, word)?)?;
        let uuid = env.new_uuid(uuid)?;

        Ok(uuid.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_UserDict_rsUpdateWord<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    uuid: JObject<'local>,
    word: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.UserDict.handle` must correspond to
            //   `Arc<voicevox_core::blocking::UserDict>`.
            env.get_rust_field::<_, _, Arc<voicevox_core::blocking::UserDict>>(&this, "handle")
        }?
        .clone();

        let uuid = env.get_uuid(&uuid)?;

        internal.update_word(uuid, word_from_java(env, word)?)?;

        Ok(())
    })
}

fn word_from_java<'local>(
    env: &mut JNIEnv<'local>,
    word: JObject<'local>,
) -> JavaApiResult<voicevox_core::UserDictWord> {
    let surface = &env
        .get_field(&word, "surface", "Ljava/lang/String;")?
        .l()?
        .into();
    let surface = &String::from(env.get_string(surface)?);

    let pronunciation = &env
        .get_field(&word, "pronunciation", "Ljava/lang/String;")?
        .l()?
        .into();
    let pronunciation = String::from(env.get_string(pronunciation)?);

    let accent_type = env
        .get_field(&word, "accentType", "I")?
        .i()?
        .try_into()
        .expect("should be validated");

    let word_type = env
        .get_field(
            &word,
            "wordType",
            "Ljp/hiroshiba/voicevoxcore/UserDictWord$Type;",
        )?
        .l()?;
    let word_type = &env
        .get_field(word_type, "identifier", "Ljava/lang/String;")?
        .l()?
        .into();
    let word_type = &String::from(env.get_string(word_type)?);
    let word_type = serde_json::from_value(json!(word_type)).expect("unknown `UserDictWordType`");

    let priority = env
        .get_field(&word, "priority", "I")?
        .i()?
        .try_into()
        .expect("should be validated");

    voicevox_core::UserDictWord::builder()
        .word_type(word_type)
        .priority(priority)
        .build(surface, pronunciation, accent_type)
        .map_err(Into::into)
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_UserDict_rsRemoveWord<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    uuid: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.UserDict.handle` must correspond to
            //   `Arc<voicevox_core::blocking::UserDict>`.
            env.get_rust_field::<_, _, Arc<voicevox_core::blocking::UserDict>>(&this, "handle")
        }?
        .clone();

        let uuid = env.get_uuid(&uuid)?;
        internal.remove_word(uuid)?;

        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_UserDict_rsImportDict<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    other_dict: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.UserDict.handle` must correspond to
            //   `Arc<voicevox_core::blocking::UserDict>`.
            env.get_rust_field::<_, _, Arc<voicevox_core::blocking::UserDict>>(&this, "handle")
        }?
        .clone();

        let other_dict = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.UserDict.handle` must correspond to
            //   `Arc<voicevox_core::blocking::UserDict>`.
            env.get_rust_field::<_, _, Arc<voicevox_core::blocking::UserDict>>(
                &other_dict,
                "handle",
            )
        }?
        .clone();

        internal.import(&other_dict)?;

        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_UserDict_rsLoad<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    path: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.UserDict.handle` must correspond to
            //   `Arc<voicevox_core::blocking::UserDict>`.
            env.get_rust_field::<_, _, Arc<voicevox_core::blocking::UserDict>>(&this, "handle")
        }?
        .clone();

        let path = env.get_string(&path)?;
        let path = &*Cow::from(&path);

        internal.load(path)?;

        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_UserDict_rsSave<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    path: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.UserDict.handle` must correspond to
            //   `Arc<voicevox_core::blocking::UserDict>`.
            env.get_rust_field::<_, _, Arc<voicevox_core::blocking::UserDict>>(&this, "handle")
        }?
        .clone();

        let path = env.get_string(&path)?;
        let path = &*Cow::from(&path);

        internal.save(path)?;

        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_UserDict_rsToHashMap<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.UserDict.handle` must correspond to
            //   `Arc<voicevox_core::blocking::UserDict>`.
            env.get_rust_field::<_, _, Arc<voicevox_core::blocking::UserDict>>(&this, "handle")
        }?
        .clone();

        let map = env.new_object("java/util/HashMap", "()V", &[])?;

        internal.with_words(|words| {
            for (&uuid, word) in words {
                let uuid = &env.new_uuid(uuid)?;
                let word = &env.new_object(
                    "jp/hiroshiba/voicevoxcore/UserDictWord",
                    "(Ljava/lang/String;Ljava/lang/String;I)V",
                    &[
                        (&env.new_string(word.surface())?).into(),
                        (&env.new_string(word.pronunciation())?).into(),
                        i32::try_from(word.accent_type())
                            .expect("should be validated")
                            .into(),
                    ],
                )?;
                env.call_method(
                    &map,
                    "put",
                    "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                    &[uuid.into(), word.into()],
                )?;
            }
            Ok::<_, jni::errors::Error>(())
        })?;

        Ok(map.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_UserDict_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.UserDict.handle` must correspond to
            //   `Arc<voicevox_core::blocking::UserDict>`.
            env.take_rust_field::<_, _, Arc<voicevox_core::blocking::UserDict>>(&this, "handle")
        }?;
        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDictWord_rsToZenkaku<'local>(
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

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_UserDictWord_rsValidatePronunciation<'local>(
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
