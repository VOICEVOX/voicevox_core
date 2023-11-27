use std::{borrow::Cow, sync::Arc};

use crate::common::{throw_if_err, RUNTIME};
use jni::{
    objects::{JObject, JString},
    JNIEnv,
};

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_OpenJtalk_rsNew<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    open_jtalk_dict_dir: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let open_jtalk_dict_dir = env.get_string(&open_jtalk_dict_dir)?;
        let open_jtalk_dict_dir = &*Cow::from(&open_jtalk_dict_dir);

        let internal = RUNTIME.block_on(voicevox_core::OpenJtalk::new(open_jtalk_dict_dir))?;
        env.set_rust_field(&this, "handle", internal)?;

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_OpenJtalk_rsUseUserDict<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    user_dict: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = env
            .get_rust_field::<_, _, voicevox_core::OpenJtalk>(&this, "handle")?
            .clone();

        let user_dict = env
            .get_rust_field::<_, _, Arc<voicevox_core::UserDict>>(&user_dict, "handle")?
            .clone();

        RUNTIME.block_on(internal.use_user_dict(&user_dict))?;

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_OpenJtalk_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        env.take_rust_field(&this, "handle")?;
        Ok(())
    })
}
