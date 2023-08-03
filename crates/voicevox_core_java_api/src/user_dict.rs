use jni::objects::JClass;
use std::sync::{Arc, Mutex};

use crate::common::throw_if_err;
use jni::{
    objects::{JObject, JString},
    sys::jobject,
    JNIEnv,
};

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_UserDict_rsNew<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = voicevox_core::UserDict::new();

        unsafe { env.set_rust_field(&this, "internal", Arc::new(Mutex::new(internal))) }?;

        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_UserDict_rsAddWord<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    word_json: JString<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::UserDict>>>(&this, "internal")?
                .clone()
        };

        let word_json = env.get_string(&word_json)?;
        let word_json = word_json.to_str()?;

        let word: voicevox_core::UserDictWord = serde_json::from_str(word_json)?;

        let uuid = {
            let mut internal = internal.lock().unwrap();
            internal.add_word(word)?
        };

        let uuid = uuid.hyphenated().to_string();
        let uuid = env.new_string(uuid)?;

        Ok(uuid.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_UserDict_rsUpdateWord<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    uuid: JString<'local>,
    word_json: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::UserDict>>>(&this, "internal")?
                .clone()
        };

        let uuid = env.get_string(&uuid)?;
        let uuid = uuid.to_str()?.parse()?;
        let word_json = env.get_string(&word_json)?;
        let word_json = word_json.to_str()?;

        let word: voicevox_core::UserDictWord = serde_json::from_str(word_json)?;

        {
            let mut internal = internal.lock().unwrap();
            internal.update_word(uuid, word)?;
        };

        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_UserDict_rsRemoveWord<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    uuid: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::UserDict>>>(&this, "internal")?
                .clone()
        };

        let uuid = env.get_string(&uuid)?;
        let uuid = uuid.to_str()?.parse()?;

        {
            let mut internal = internal.lock().unwrap();
            internal.remove_word(uuid)?;
        };

        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_UserDict_rsImportDict<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    other_dict: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::UserDict>>>(&this, "internal")?
                .clone()
        };
        let other_dict = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::UserDict>>>(
                &other_dict,
                "internal",
            )?
            .clone()
        };

        {
            let mut internal = internal.lock().unwrap();
            let mut other_dict = other_dict.lock().unwrap();
            internal.import(&mut other_dict)?;
        }

        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_UserDict_rsLoad<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    path: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::UserDict>>>(&this, "internal")?
                .clone()
        };

        let path = env.get_string(&path)?;
        let path = path.to_str()?;

        {
            let mut internal = internal.lock().unwrap();
            internal.load(path)?;
        };

        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_UserDict_rsSave<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    path: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::UserDict>>>(&this, "internal")?
                .clone()
        };

        let path = env.get_string(&path)?;
        let path = path.to_str()?;

        {
            let internal = internal.lock().unwrap();
            internal.save(path)?;
        };

        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_UserDict_rsGetWords<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::UserDict>>>(&this, "internal")?
                .clone()
        };

        let words = {
            let internal = internal.lock().unwrap();
            serde_json::to_string(internal.words())?
        };

        let words = env.new_string(words)?;

        Ok(words.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_UserDict_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal =
            unsafe { env.get_rust_field::<_, _, voicevox_core::VoiceModel>(&this, "internal") }?;
        drop(internal);
        unsafe { env.take_rust_field(&this, "internal") }?;
        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_UserDict_rsToZenkaku<'local>(
    env: JNIEnv<'local>,
    _cls: JClass<'local>,
    text: JString<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let text = env.get_string(&text)?;
        let text = text.to_str()?;

        let text = voicevox_core::to_zenkaku(text);

        let text = env.new_string(text)?;
        Ok(text.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_UserDict_rsValidatePronunciation<'local>(
    env: JNIEnv<'local>,
    _cls: JClass<'local>,
    text: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let text = env.get_string(&text)?;
        let text = text.to_str()?;

        voicevox_core::validate_pronunciation(text)?;

        Ok(())
    })
}
