use crate::{
    common::{throw_if_err, RUNTIME},
    enum_object, object, object_type,
};

use anyhow::anyhow;
use jni::{objects::JObject, JNIEnv};
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
        unsafe { env.set_rust_field(&this, "internal", Mutex::new(internal))? };
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
            env.get_rust_field::<_, _, Mutex<voicevox_core::Synthesizer>>(&this, "internal")?
        };
        let mut internal = internal.lock().unwrap();
        RUNTIME.block_on(internal.load_voice_model(&model))?;
        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_Synthesizer_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Mutex<voicevox_core::Synthesizer>>(&this, "internal")
        }?;
        drop(internal);
        unsafe { env.take_rust_field(&this, "internal") }?;
        Ok(())
    })
}
