use std::{borrow::Cow, sync::Arc};

use crate::common::{throw_if_err, Closable, HasJavaClassIdent, JNIEnvExt as _};
use jni::{
    objects::{JObject, JString},
    sys::jobject,
    JNIEnv,
};

pub(crate) type VoiceModelFile = Arc<Closable<voicevox_core::blocking::VoiceModelFile>>;

impl HasJavaClassIdent for voicevox_core::blocking::VoiceModelFile {
    const JAVA_CLASS_IDENT: &str = "VoiceModelFile";
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_VoiceModelFile_rsOpen<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model_path: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let model_path = env.get_string(&model_path)?;
        let model_path = &*Cow::from(&model_path);

        let internal = voicevox_core::blocking::VoiceModelFile::open(model_path)?;
        let internal = Arc::new(Closable::new(internal));
        env.set_rust_field(&this, "handle", internal)?;

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_VoiceModelFile_rsGetId<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = env
            .get_rust_field::<_, _, VoiceModelFile>(&this, "handle")?
            .clone();
        let internal = internal.read()?;

        let id = env.new_uuid(internal.id().raw_voice_model_id())?;

        Ok(id.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_VoiceModelFile_rsGetMetasJson<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = env
            .get_rust_field::<_, _, VoiceModelFile>(&this, "handle")?
            .clone();
        let internal = internal.read()?;

        let metas = internal.metas();
        let metas_json = serde_json::to_string(&metas).expect("should not fail");
        Ok(env.new_string(metas_json)?.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_VoiceModelFile_rsClose<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        env.take_rust_field::<_, _, VoiceModelFile>(&this, "handle")?
            .close();
        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_VoiceModelFile_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        env.take_rust_field(&this, "handle")?;
        Ok(())
    })
}
