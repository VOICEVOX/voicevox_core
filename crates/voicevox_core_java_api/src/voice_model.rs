use std::sync::Arc;

use crate::common::{throw_if_err, RUNTIME};
use jni::{
    objects::{JObject, JString},
    sys::jobject,
    JNIEnv,
};

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_VoiceModel_rsFromPath<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model_path: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let model_path = env.get_string(&model_path)?;
        let model_path = model_path.to_str()?;

        let internal = RUNTIME.block_on(voicevox_core::VoiceModel::from_path(model_path))?;

        env.set_rust_field(&this, "handle", Arc::new(internal))?;

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_VoiceModel_rsGetId<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::VoiceModel>>(&this, "handle")?
            .clone();

        let id = internal.id().raw_voice_model_id();

        let id = env.new_string(id)?;

        Ok(id.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_VoiceModel_rsGetMetasJson<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::VoiceModel>>(&this, "handle")?
            .clone();

        let metas = internal.metas();
        let metas_json = serde_json::to_string(&metas)?;
        Ok(env.new_string(metas_json)?.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_VoiceModel_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        env.take_rust_field(&this, "handle")?;
        Ok(())
    })
}
