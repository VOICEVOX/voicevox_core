use std::{borrow::Cow, ptr, sync::Arc};

use crate::common::throw_if_err;
use jni::{
    JNIEnv,
    objects::{JObject, JString},
    sys::jstring,
};
use voicevox_core::__internal::interop::BlockingTextAnalyzerExt as _;

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_OpenJtalk_rsNew<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    open_jtalk_dict_dir: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let open_jtalk_dict_dir = env.get_string(&open_jtalk_dict_dir)?;
        let open_jtalk_dict_dir = &*Cow::from(&open_jtalk_dict_dir);

        let internal = voicevox_core::blocking::OpenJtalk::new(open_jtalk_dict_dir)?;

        // SAFETY:
        // - The safety contract must be upheld by the caller.
        // - `jp.hiroshiba.voicevoxcore.blocking.OpenJtalk.handle` must correspond to
        //   `voicevox_core::blocking::OpenJtalk`.
        unsafe { env.set_rust_field(&this, "handle", internal) }?;

        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_OpenJtalk_rsUseUserDict<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    user_dict: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.OpenJtalk.handle` must correspond to
            //   `voicevox_core::blocking::OpenJtalk`.
            env.get_rust_field::<_, _, voicevox_core::blocking::OpenJtalk>(&this, "handle")
        }?
        .clone();

        let user_dict = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.UserDict.handle` must correspond to
            //   `Arc<voicevox_core::blocking::UserDict>`.
            env.get_rust_field::<_, _, Arc<voicevox_core::blocking::UserDict>>(&user_dict, "handle")
        }?
        .clone();

        internal.use_user_dict(&user_dict)?;

        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_OpenJtalk_rsAnalyze<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    text: JString<'local>,
) -> jstring {
    throw_if_err(env, ptr::null_mut(), |env| {
        let text = &String::from(env.get_string(&text)?);
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.OpenJtalk.handle` must correspond to
            //   `voicevox_core::blocking::OpenJtalk`.
            env.get_rust_field::<_, _, voicevox_core::blocking::OpenJtalk>(&this, "handle")
        }?
        .clone();
        let accent_phrases = &internal.analyze_(text)?;
        let accent_phrases = serde_json::to_string(accent_phrases).expect("should not fail");
        let accent_phrases = env.new_string(accent_phrases)?;
        Ok(accent_phrases.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_OpenJtalk_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.OpenJtalk.handle` must correspond to
            //   `voicevox_core::blocking::OpenJtalk`.
            env.take_rust_field::<_, _, voicevox_core::blocking::OpenJtalk>(&this, "handle")
        }?;
        Ok(())
    })
}
