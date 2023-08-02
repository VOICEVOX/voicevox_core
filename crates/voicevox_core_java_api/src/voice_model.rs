use std::sync::Arc;

use crate::{
    common::{throw_if_err, RUNTIME},
    object_type,
};
use jni::{
    objects::{JObject, JString, JValueGen},
    sys::jobject,
    JNIEnv,
};

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_VoiceModel_rsFromPath<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model_path: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let model_path = env.get_string(&model_path)?;
        let model_path = model_path.to_str()?;

        let internal = RUNTIME.block_on(voicevox_core::VoiceModel::from_path(model_path))?;

        unsafe { env.set_rust_field(&this, "internal", Arc::new(internal)) }?;

        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_VoiceModel_rsGetId<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<voicevox_core::VoiceModel>>(&this, "internal")
        }?
        .clone();

        let id = internal.id().raw_voice_model_id();

        let id = env.new_string(id)?;

        Ok(id.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_VoiceModel_rsGetMetas<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<voicevox_core::VoiceModel>>(&this, "internal")
        }?
        .clone();
        let j_speakers = env.new_object_array(
            internal.metas().len() as i32,
            object_type!("VoiceModel$SpeakerMeta"),
            JObject::null(),
        )?;
        for (i, meta) in internal.metas().iter().enumerate() {
            let j_styles = env.new_object_array(
                meta.styles().len() as i32,
                object_type!("VoiceModel$StyleMeta"),
                JObject::null(),
            )?;
            for (j, style) in meta.styles().iter().enumerate() {
                let j_style = env.new_object(
                    object_type!("VoiceModel$StyleMeta"),
                    concat!(
                        "(",
                        "Ljava/lang/String;", // name
                        "I",                  // id
                        ")V"
                    ),
                    &[
                        JValueGen::Object(&env.new_string(style.name())?.into()),
                        JValueGen::Int(style.id().raw_id() as i32),
                    ],
                )?;
                env.set_object_array_element(&j_styles, j as i32, j_style)?;
            }

            let j_meta = env.new_object(
                object_type!("VoiceModel$SpeakerMeta"),
                concat!(
                    "(",
                    "Ljava/lang/String;", // name
                    "[",                  // styles
                    object_type!("VoiceModel$StyleMeta"),
                    "Ljava/lang/String;", // speakerUuid
                    "Ljava/lang/String;", // version
                    ")V"
                ),
                &[
                    JValueGen::Object(&env.new_string(meta.name())?.into()),
                    JValueGen::Object(&j_styles.into()),
                    JValueGen::Object(&env.new_string(meta.speaker_uuid())?.into()),
                    JValueGen::Object(&env.new_string(meta.version().raw_version())?.into()),
                ],
            )?;
            env.set_object_array_element(&j_speakers, i as i32, j_meta)?;
        }
        Ok(j_speakers.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_VoiceModel_rsDrop<'local>(
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
