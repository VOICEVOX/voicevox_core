use std::sync::{Arc, Mutex};

use crate::common::throw_if_err;
use jni::{
    objects::{JObject, JString},
    JNIEnv,
};
#[no_mangle]
unsafe extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_rsNewWithoutDic<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = voicevox_core::OpenJtalk::new_without_dic();

        env.set_rust_field(&this, "handle", Arc::new(internal))?;
        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_rsNewWithInitialize<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    open_jtalk_dict_dir: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let open_jtalk_dict_dir = env.get_string(&open_jtalk_dict_dir)?;
        let open_jtalk_dict_dir = open_jtalk_dict_dir.to_str()?;

        let internal = voicevox_core::OpenJtalk::new_with_initialize(open_jtalk_dict_dir)?;
        env.set_rust_field(&this, "handle", Arc::new(internal))?;

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_rsUseUserDict<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    user_dict: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::OpenJtalk>>(&this, "handle")?
            .clone();

        let user_dict = env
            .get_rust_field::<_, _, Arc<Mutex<voicevox_core::UserDict>>>(&user_dict, "handle")?
            .clone();

        {
            let user_dict = user_dict.lock().unwrap();
            internal.use_user_dict(&user_dict)?
        }

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        env.take_rust_field(&this, "handle")?;
        Ok(())
    })
}
