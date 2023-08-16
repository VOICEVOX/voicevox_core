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
            env.get_rust_field::<_, _, Arc<voicevox_core::OpenJtalk>>(&open_jtalk, "handle")?
                .clone()
        };
        let internal = RUNTIME.block_on(voicevox_core::Synthesizer::new_with_initialize(
            open_jtalk,
            Box::leak(Box::new(options)),
        ))?;
        unsafe { env.set_rust_field(&this, "handle", Arc::new(Mutex::new(internal)))? };
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
            env.get_rust_field::<_, _, Arc<voicevox_core::VoiceModel>>(&model, "handle")?
                .clone()
        };
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "handle")?
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
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "handle")?
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
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "handle")?
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
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "handle")?
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
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "handle")?
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
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsReplaceMoraData<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    accent_phrases_json: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let accent_phrases_json: String = env.get_string(&accent_phrases_json)?.into();
        let accent_phrases: Vec<voicevox_core::AccentPhraseModel> =
            serde_json::from_str(&accent_phrases_json)?;
        let style_id = style_id as u32;

        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "handle")?
                .clone()
        };

        let replaced_accent_phrases = {
            let internal = internal.lock().unwrap();
            RUNTIME.block_on(
                internal.replace_mora_data(&accent_phrases, voicevox_core::StyleId::new(style_id)),
            )?
        };

        let replaced_accent_phrases_json = serde_json::to_string(&replaced_accent_phrases)?;

        Ok(env.new_string(replaced_accent_phrases_json)?.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsReplacePhonemeLength<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    accent_phrases_json: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let accent_phrases_json: String = env.get_string(&accent_phrases_json)?.into();
        let accent_phrases: Vec<voicevox_core::AccentPhraseModel> =
            serde_json::from_str(&accent_phrases_json)?;
        let style_id = style_id as u32;

        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "handle")?
                .clone()
        };

        let replaced_accent_phrases = {
            let internal = internal.lock().unwrap();
            RUNTIME.block_on(
                internal
                    .replace_phoneme_length(&accent_phrases, voicevox_core::StyleId::new(style_id)),
            )?
        };

        let replaced_accent_phrases_json = serde_json::to_string(&replaced_accent_phrases)?;

        Ok(env.new_string(replaced_accent_phrases_json)?.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsReplaceMoraPitch<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    accent_phrases_json: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let accent_phrases_json: String = env.get_string(&accent_phrases_json)?.into();
        let accent_phrases: Vec<voicevox_core::AccentPhraseModel> =
            serde_json::from_str(&accent_phrases_json)?;
        let style_id = style_id as u32;

        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "handle")?
                .clone()
        };

        let replaced_accent_phrases = {
            let internal = internal.lock().unwrap();
            RUNTIME.block_on(
                internal.replace_mora_pitch(&accent_phrases, voicevox_core::StyleId::new(style_id)),
            )?
        };

        let replaced_accent_phrases_json = serde_json::to_string(&replaced_accent_phrases)?;

        Ok(env.new_string(replaced_accent_phrases_json)?.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsSynthesis<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    query_json: JString<'local>,
    style_id: jint,
    enable_interrogative_upspeak: jboolean,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let audio_query: String = env.get_string(&query_json)?.into();
        let audio_query: voicevox_core::AudioQueryModel = serde_json::from_str(&audio_query)?;
        let style_id = style_id as u32;

        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "handle")?
                .clone()
        };

        let wave = {
            let internal = internal.lock().unwrap();
            let options = voicevox_core::SynthesisOptions {
                enable_interrogative_upspeak: enable_interrogative_upspeak != 0,
                // ..Default::default()
            };
            RUNTIME.block_on(internal.synthesis(
                &audio_query,
                voicevox_core::StyleId::new(style_id),
                &options,
            ))?
        };

        let j_bytes = env.byte_array_from_slice(&wave)?;

        Ok(j_bytes.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsTts<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    query_json: JString<'local>,
    style_id: jint,
    kana: jboolean,
    enable_interrogative_upspeak: jboolean,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let text: String = env.get_string(&query_json)?.into();
        let style_id = style_id as u32;

        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<Mutex<voicevox_core::Synthesizer>>>(&this, "handle")?
                .clone()
        };

        let wave = {
            let internal = internal.lock().unwrap();
            let options = voicevox_core::TtsOptions {
                kana: kana != 0,
                enable_interrogative_upspeak: enable_interrogative_upspeak != 0,
                // ..Default::default()
            };
            RUNTIME.block_on(internal.tts(
                &text,
                voicevox_core::StyleId::new(style_id),
                &options,
            ))?
        };

        let j_bytes = env.byte_array_from_slice(&wave)?;

        Ok(j_bytes.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        unsafe { env.take_rust_field(&this, "handle") }?;
        Ok(())
    })
}
