use std::sync::Arc;

use crate::{
    common::{throw_if_err, RUNTIME},
    object_type,
};
use jni::{
    objects::{JObject, JString, JValueGen},
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

        env.set_field(
            &this,
            "id",
            "Ljava/lang/String;",
            JValueGen::Object(&env.new_string(internal.id().raw_voice_model_id())?.into()),
        )?;
        let speakers = env.new_object_array(
            internal.metas().len() as i32,
            object_type!("VoiceModel$SpeakerMeta"),
            JObject::null(),
        )?;
        for (i, meta) in internal.metas().iter().enumerate() {
            let j_meta = env.new_object(object_type!("VoiceModel$SpeakerMeta"), "()V", &[])?;
            env.set_field(
                &j_meta,
                "name",
                "Ljava/lang/String;",
                JValueGen::Object(&env.new_string(meta.name())?.into()),
            )?;
            let j_styles = env.new_object_array(
                meta.styles().len() as i32,
                object_type!("VoiceModel$StyleMeta"),
                JObject::null(),
            )?;
            for (j, style) in meta.styles().iter().enumerate() {
                let j_style = env.new_object(object_type!("VoiceModel$StyleMeta"), "()V", &[])?;
                env.set_field(
                    &j_style,
                    "name",
                    "Ljava/lang/String;",
                    JValueGen::Object(&env.new_string(style.name())?.into()),
                )?;
                env.set_field(
                    &j_style,
                    "id",
                    "I",
                    JValueGen::Int(style.id().raw_id() as i32),
                )?;
                env.set_object_array_element(&j_styles, j as i32, j_style)?;
            }
            env.set_field(
                &j_meta,
                "styles",
                concat!("[", object_type!("VoiceModel$StyleMeta")),
                JValueGen::Object(&j_styles),
            )?;
            env.set_field(
                &j_meta,
                "speakerUuid",
                "Ljava/lang/String;",
                JValueGen::Object(&env.new_string(meta.speaker_uuid())?.into()),
            )?;
            env.set_field(
                &j_meta,
                "version",
                "Ljava/lang/String;",
                JValueGen::Object(&env.new_string(meta.version().raw_version())?.into()),
            )?;

            env.set_object_array_element(&speakers, i as i32, j_meta)?;
        }
        unsafe { env.set_rust_field(&this, "internal", Arc::new(internal)) }?;

        Ok(())
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
