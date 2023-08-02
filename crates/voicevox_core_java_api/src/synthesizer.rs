use crate::{
    common::{throw_if_err, RUNTIME},
    enum_object, object, object_type,
};

use anyhow::anyhow;
use jni::{
    objects::{JObject, JString},
    sys::{jboolean, jint, jobject},
    JNIEnv,
};
use std::sync::{Arc, Mutex};

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsNewWithInitialize<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    open_jtalk: JObject<'local>,
    builder: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let mut options = voicevox_core::InitializeOptions::default();

        let acceleration_mode = env
            .get_field(
                &builder,
                "accelerationMode",
                object_type!("Synthesizer$AccelerationMode"),
            )?
            .l()?;

        if !acceleration_mode.is_null() {
            let auto = enum_object!(env, "Synthesizer$AccelerationMode", "AUTO")?;
            let cpu = enum_object!(env, "Synthesizer$AccelerationMode", "CPU")?;
            let gpu = enum_object!(env, "Synthesizer$AccelerationMode", "GPU")?;
            options.acceleration_mode = if env.is_same_object(&acceleration_mode, auto)? {
                voicevox_core::AccelerationMode::Auto
            } else if env.is_same_object(&acceleration_mode, cpu)? {
                voicevox_core::AccelerationMode::Cpu
            } else if env.is_same_object(&acceleration_mode, gpu)? {
                voicevox_core::AccelerationMode::Gpu
            } else {
                return Err(anyhow!("invalid acceleration mode".to_string(),));
            };
        }
        let cpu_num_threads = env.get_field(&builder, "cpuNumThreads", "I")?;
        if let Ok(cpu_num_threads) = cpu_num_threads.i() {
            options.cpu_num_threads = cpu_num_threads as u16;
        }

        let load_all_models = env.get_field(&builder, "loadAllModels", "Z")?;
        if let Ok(load_all_models) = load_all_models.z() {
            options.load_all_models = load_all_models;
        }

        let open_jtalk = unsafe {
            env.get_rust_field::<_, _, Arc<voicevox_core::OpenJtalk>>(&open_jtalk, "internal")?
                .clone()
        };
        let internal = RUNTIME.block_on(voicevox_core::Synthesizer::new_with_initialize(
            open_jtalk.clone(),
            Box::leak(Box::new(options)),
        ))?;
        unsafe { env.set_rust_field(&this, "internal", Arc::new(Mutex::new(internal)))? };
        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsLoadVoiceModel<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let model = unsafe {
            env.get_rust_field::<_, _, Arc<voicevox_core::VoiceModel>>(&model, "internal")?
                .clone()
        };
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "internal")?
                .clone()
        };
        {
            let mut internal = internal.lock().unwrap();
            RUNTIME.block_on(internal.load_voice_model(&model))?;
        }
        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsUnloadVoiceModel<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model_id: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let model_id: String = env.get_string(&model_id)?.into();

        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "internal")?
                .clone()
        };

        {
            let mut internal = internal.lock().unwrap();

            internal.unload_voice_model(&voicevox_core::VoiceModelId::new(model_id))?;
        }

        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsIsLoadedVoiceModel<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model_id: JString<'local>,
) -> jboolean {
    throw_if_err(env, false, |env| {
        let model_id: String = env.get_string(&model_id)?.into();

        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "internal")?
                .clone()
        };

        let is_loaded = {
            let internal = internal.lock().unwrap();
            internal.is_loaded_voice_model(&voicevox_core::VoiceModelId::new(model_id))
        };

        Ok(is_loaded)
    }) as jboolean
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsAudioQuery<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    text: JString<'local>,
    style_id: jint,
    kana: jboolean,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let text: String = env.get_string(&text)?.into();
        let style_id = style_id as u32;

        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "internal")?
                .clone()
        };

        let audio_query = {
            let internal = internal.lock().unwrap();
            let options = voicevox_core::AudioQueryOptions {
                kana: kana != 0,
                // ..Default::default()
            };
            RUNTIME.block_on(internal.audio_query(
                &text,
                voicevox_core::StyleId::new(style_id),
                &options,
            ))?
        };

        let query_json = serde_json::to_string(&audio_query)?;

        let j_audio_query = env.new_string(query_json)?;

        Ok(j_audio_query.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsAccentPhrases<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    text: JString<'local>,
    style_id: jint,
    kana: jboolean,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let text: String = env.get_string(&text)?.into();
        let style_id = style_id as u32;

        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "internal")?
                .clone()
        };

        let accent_phrases = {
            let internal = internal.lock().unwrap();
            let options = voicevox_core::AccentPhrasesOptions {
                kana: kana != 0,
                // ..Default::default()
            };
            RUNTIME.block_on(internal.create_accent_phrases(
                &text,
                voicevox_core::StyleId::new(style_id),
                &options,
            ))?
        };

        let query_json = serde_json::to_string(&accent_phrases)?;

        let j_accent_phrases = env.new_string(query_json)?;

        Ok(j_accent_phrases.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "internal")?
                .clone()
        };
        drop(internal);
        unsafe { env.take_rust_field(&this, "internal") }?;
        Ok(())
    })
}
