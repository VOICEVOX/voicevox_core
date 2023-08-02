use std::sync::Arc;

use crate::common::throw_if_err;
use jni::{
    objects::{JObject, JString},
    JNIEnv,
};
#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_rsNewWithoutDic<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = voicevox_core::OpenJtalk::new_without_dic();

        unsafe { env.set_rust_field(&this, "internal", Arc::new(internal)) }?;
        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_rsNewWithInitialize<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    open_jtalk_dict_dir: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let open_jtalk_dict_dir = env.get_string(&open_jtalk_dict_dir)?;
        let open_jtalk_dict_dir = open_jtalk_dict_dir.to_str()?;

        let internal = voicevox_core::OpenJtalk::new_with_initialize(open_jtalk_dict_dir)?;
        unsafe { env.set_rust_field(&this, "internal", Arc::new(internal)) }?;

        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<voicevox_core::OpenJtalk>>(&this, "internal")
        }?;
        drop(internal);
        unsafe { env.take_rust_field(&this, "internal") }?;
        Ok(())
    })
}
