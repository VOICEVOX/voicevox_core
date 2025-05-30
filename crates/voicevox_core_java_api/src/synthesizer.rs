use crate::{
    common::{JNIEnvExt as _, JavaApiError, throw_if_err},
    object, object_type, static_field,
};

use jni::{
    JNIEnv,
    objects::{JObject, JString},
    sys::{jboolean, jint, jobject},
};
use std::sync::Arc;

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsNew<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    onnxruntime: JObject<'local>,
    open_jtalk: JObject<'local>,
    builder: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let acceleration_mode = env
            .get_field(
                &builder,
                "accelerationMode",
                object_type!("AccelerationMode"),
            )?
            .l()?;

        let acceleration_mode = if acceleration_mode.is_null() {
            Default::default()
        } else {
            let auto = static_field!(env, "AccelerationMode", "AUTO")?;
            let cpu = static_field!(env, "AccelerationMode", "CPU")?;
            let gpu = static_field!(env, "AccelerationMode", "GPU")?;
            if env.is_same_object(&acceleration_mode, auto)? {
                voicevox_core::AccelerationMode::Auto
            } else if env.is_same_object(&acceleration_mode, cpu)? {
                voicevox_core::AccelerationMode::Cpu
            } else if env.is_same_object(&acceleration_mode, gpu)? {
                voicevox_core::AccelerationMode::Gpu
            } else {
                panic!("予期しない`AccelerationMode`です: {acceleration_mode:?}");
            }
        };
        let cpu_num_threads = env
            .get_field(&builder, "cpuNumThreads", "I")?
            .i()
            .expect("cpuNumThreads is not integer") as u16;

        let onnxruntime = *unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Onnxruntime.handle` must correspond to
            //   `&'static voicevox_core::blocking::Onnxruntime`.
            env.get_rust_field::<_, _, &'static voicevox_core::blocking::Onnxruntime>(
                &onnxruntime,
                "handle",
            )
        }?;
        let open_jtalk = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.OpenJtalk.handle` must correspond to
            //   `voicevox_core::blocking::OpenJtalk`.
            env.get_rust_field::<_, _, voicevox_core::blocking::OpenJtalk>(&open_jtalk, "handle")
        }?
        .clone();
        let internal = Arc::new(
            voicevox_core::blocking::Synthesizer::builder(onnxruntime)
                .text_analyzer(open_jtalk)
                .acceleration_mode(acceleration_mode)
                .cpu_num_threads(cpu_num_threads)
                .build()?,
        );
        // SAFETY:
        // - The safety contract must be upheld by the caller.
        // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
        //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
        unsafe { env.set_rust_field(&this, "handle", internal) }?;
        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsIsGpuMode<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jboolean {
    throw_if_err(env, false, |env| {
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            env.get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )
        }?
        .clone();

        Ok(internal.is_gpu_mode())
    })
    .into()
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsGetMetasJson<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        let metas_json = serde_json::to_string(&internal.metas()).expect("should not fail");

        let j_metas_json = env.new_string(metas_json)?;

        Ok(j_metas_json.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsLoadVoiceModel<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let model = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.VoiceModelFile.handle` must correspond to
            //   `crate::voice_model::VoiceModelFile`.
            env.get_rust_field::<_, _, crate::voice_model::VoiceModelFile>(&model, "handle")
        }?
        .clone();
        let model = model.read()?;
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();
        internal.load_voice_model(&model)?;
        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsUnloadVoiceModel<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model_id: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let model_id = env.get_uuid(&model_id)?.into();

        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        internal.unload_voice_model(model_id)?;

        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsIsLoadedVoiceModel<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model_id: JObject<'local>,
) -> jboolean {
    throw_if_err(env, false, |env| {
        let model_id = env.get_uuid(&model_id)?.into();

        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        let is_loaded = internal.is_loaded_voice_model(model_id);

        Ok(is_loaded)
    })
    .into()
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsCreateAudioQueryFromKana<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    kana: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let kana: String = env.get_string(&kana)?.into();
        let style_id = style_id as u32;

        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        let audio_query =
            internal.create_audio_query_from_kana(&kana, voicevox_core::StyleId::new(style_id))?;

        let query_json = serde_json::to_string(&audio_query).expect("should not fail");

        let j_audio_query = env.new_string(query_json)?;

        Ok(j_audio_query.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsCreateAudioQuery<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    text: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let text: String = env.get_string(&text)?.into();
        let style_id = style_id as u32;

        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        let audio_query =
            internal.create_audio_query(&text, voicevox_core::StyleId::new(style_id))?;

        let query_json = serde_json::to_string(&audio_query).expect("should not fail");

        let j_audio_query = env.new_string(query_json)?;

        Ok(j_audio_query.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsAccentPhrasesFromKana<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    kana: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let kana: String = env.get_string(&kana)?.into();
        let style_id = style_id as u32;

        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        let accent_phrases = internal
            .create_accent_phrases_from_kana(&kana, voicevox_core::StyleId::new(style_id))?;

        let query_json = serde_json::to_string(&accent_phrases).expect("should not fail");

        let j_accent_phrases = env.new_string(query_json)?;

        Ok(j_accent_phrases.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsAccentPhrases<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    text: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let text: String = env.get_string(&text)?.into();
        let style_id = style_id as u32;

        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        let accent_phrases =
            internal.create_accent_phrases(&text, voicevox_core::StyleId::new(style_id))?;

        let query_json = serde_json::to_string(&accent_phrases).expect("should not fail");

        let j_accent_phrases = env.new_string(query_json)?;

        Ok(j_accent_phrases.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsReplaceMoraData<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    accent_phrases_json: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let accent_phrases_json: String = env.get_string(&accent_phrases_json)?.into();
        let accent_phrases: Vec<voicevox_core::AccentPhrase> =
            serde_json::from_str(&accent_phrases_json).map_err(JavaApiError::DeJson)?;
        let style_id = style_id as u32;

        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        let replaced_accent_phrases =
            internal.replace_mora_data(&accent_phrases, voicevox_core::StyleId::new(style_id))?;

        let replaced_accent_phrases_json =
            serde_json::to_string(&replaced_accent_phrases).expect("should not fail");

        Ok(env.new_string(replaced_accent_phrases_json)?.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsReplacePhonemeLength<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    accent_phrases_json: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let accent_phrases_json: String = env.get_string(&accent_phrases_json)?.into();
        let accent_phrases: Vec<voicevox_core::AccentPhrase> =
            serde_json::from_str(&accent_phrases_json).map_err(JavaApiError::DeJson)?;
        let style_id = style_id as u32;

        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        let replaced_accent_phrases = internal
            .replace_phoneme_length(&accent_phrases, voicevox_core::StyleId::new(style_id))?;

        let replaced_accent_phrases_json =
            serde_json::to_string(&replaced_accent_phrases).expect("should not fail");

        Ok(env.new_string(replaced_accent_phrases_json)?.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsReplaceMoraPitch<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    accent_phrases_json: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let accent_phrases_json: String = env.get_string(&accent_phrases_json)?.into();
        let accent_phrases: Vec<voicevox_core::AccentPhrase> =
            serde_json::from_str(&accent_phrases_json).map_err(JavaApiError::DeJson)?;
        let style_id = style_id as u32;

        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        let replaced_accent_phrases =
            internal.replace_mora_pitch(&accent_phrases, voicevox_core::StyleId::new(style_id))?;

        let replaced_accent_phrases_json =
            serde_json::to_string(&replaced_accent_phrases).expect("should not fail");

        Ok(env.new_string(replaced_accent_phrases_json)?.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsSynthesis<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    query_json: JString<'local>,
    style_id: jint,
    enable_interrogative_upspeak: jboolean,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let audio_query: String = env.get_string(&query_json)?.into();
        let audio_query: voicevox_core::AudioQuery =
            serde_json::from_str(&audio_query).map_err(JavaApiError::DeJson)?;
        let style_id = style_id as u32;

        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        let wave = internal
            .synthesis(&audio_query, voicevox_core::StyleId::new(style_id))
            .enable_interrogative_upspeak(enable_interrogative_upspeak != 0)
            .perform()?;

        let j_bytes = env.byte_array_from_slice(&wave)?;

        Ok(j_bytes.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsTtsFromKana<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    kana: JString<'local>,
    style_id: jint,
    enable_interrogative_upspeak: jboolean,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let kana: String = env.get_string(&kana)?.into();
        let style_id = style_id as u32;

        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        let wave = internal
            .tts_from_kana(&kana, voicevox_core::StyleId::new(style_id))
            .enable_interrogative_upspeak(enable_interrogative_upspeak != 0)
            .perform()?;

        let j_bytes = env.byte_array_from_slice(&wave)?;

        Ok(j_bytes.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsTts<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    query_json: JString<'local>,
    style_id: jint,
    enable_interrogative_upspeak: jboolean,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let text: String = env.get_string(&query_json)?.into();
        let style_id = style_id as u32;

        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?
        .clone();

        let wave = internal
            .tts(&text, voicevox_core::StyleId::new(style_id))
            .enable_interrogative_upspeak(enable_interrogative_upspeak != 0)
            .perform()?;

        let j_bytes = env.byte_array_from_slice(&wave)?;

        Ok(j_bytes.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Synthesizer_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Synthesizer.handle` must correspond to
            //   `Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>`.
            type RustField =
                Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>;
            env.take_rust_field::<_, _, RustField>(&this, "handle")
        }?;
        Ok(())
    })
}
