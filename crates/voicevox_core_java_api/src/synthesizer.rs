use crate::{
    common::{throw_if_err, JavaApiError},
    enum_object, object, object_type,
};

use jni::{
    objects::{JObject, JString},
    sys::{jboolean, jdouble, jint, jobject},
    JNIEnv,
};
use std::{borrow::Cow, sync::Arc};

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsNew<'local>(
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
                panic!("予期しない`AccelerationMode`です: {acceleration_mode:?}");
            };
        }
        let cpu_num_threads = env.get_field(&builder, "cpuNumThreads", "I")?;
        options.cpu_num_threads = cpu_num_threads.i().expect("cpuNumThreads is not integer") as u16;

        let open_jtalk = env
            .get_rust_field::<_, _, voicevox_core::blocking::OpenJtalk>(&open_jtalk, "handle")?
            .clone();
        let internal = Arc::new(voicevox_core::blocking::Synthesizer::new(
            open_jtalk, &options,
        )?);
        env.set_rust_field(&this, "handle", internal)?;
        Ok(())
    })
}
#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsIsGpuMode<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jboolean {
    throw_if_err(env, false, |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        Ok(internal.is_gpu_mode())
    })
    .into()
}
#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsGetMetasJson<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let metas_json = serde_json::to_string(&internal.metas()).expect("should not fail");

        let j_metas_json = env.new_string(metas_json)?;

        Ok(j_metas_json.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsMorphableTargetsJson<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let style_id = voicevox_core::StyleId::new(style_id as _);

        let json = &internal.morphable_targets(style_id)?;
        let json = env.new_string(serde_json::to_string(json).expect("should not fail"))?;
        Ok(json.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsLoadVoiceModel<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let model = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::VoiceModel>>(&model, "handle")?
            .clone();
        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();
        internal.load_voice_model(&model)?;
        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsUnloadVoiceModel<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model_id: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let model_id: String = env.get_string(&model_id)?.into();

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        internal.unload_voice_model(&voicevox_core::VoiceModelId::new(model_id))?;

        Ok(())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsIsLoadedVoiceModel<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model_id: JString<'local>,
) -> jboolean {
    throw_if_err(env, false, |env| {
        let model_id: String = env.get_string(&model_id)?.into();

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let is_loaded = internal.is_loaded_voice_model(&voicevox_core::VoiceModelId::new(model_id));

        Ok(is_loaded)
    })
    .into()
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsAudioQueryFromKana<
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

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let audio_query =
            internal.audio_query_from_kana(&kana, voicevox_core::StyleId::new(style_id))?;

        let query_json = serde_json::to_string(&audio_query).expect("should not fail");

        let j_audio_query = env.new_string(query_json)?;

        Ok(j_audio_query.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsAudioQuery<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    text: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let text: String = env.get_string(&text)?.into();
        let style_id = style_id as u32;

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let audio_query = internal.audio_query(&text, voicevox_core::StyleId::new(style_id))?;

        let query_json = serde_json::to_string(&audio_query).expect("should not fail");

        let j_audio_query = env.new_string(query_json)?;

        Ok(j_audio_query.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsAccentPhrasesFromKana<
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

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let accent_phrases = internal
            .create_accent_phrases_from_kana(&kana, voicevox_core::StyleId::new(style_id))?;

        let query_json = serde_json::to_string(&accent_phrases).expect("should not fail");

        let j_accent_phrases = env.new_string(query_json)?;

        Ok(j_accent_phrases.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsAccentPhrases<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    text: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let text: String = env.get_string(&text)?.into();
        let style_id = style_id as u32;

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let accent_phrases =
            internal.create_accent_phrases(&text, voicevox_core::StyleId::new(style_id))?;

        let query_json = serde_json::to_string(&accent_phrases).expect("should not fail");

        let j_accent_phrases = env.new_string(query_json)?;

        Ok(j_accent_phrases.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsReplaceMoraData<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    accent_phrases_json: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let accent_phrases_json: String = env.get_string(&accent_phrases_json)?.into();
        let accent_phrases: Vec<voicevox_core::AccentPhraseModel> =
            serde_json::from_str(&accent_phrases_json).map_err(JavaApiError::DeJson)?;
        let style_id = style_id as u32;

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let replaced_accent_phrases =
            internal.replace_mora_data(&accent_phrases, voicevox_core::StyleId::new(style_id))?;

        let replaced_accent_phrases_json =
            serde_json::to_string(&replaced_accent_phrases).expect("should not fail");

        Ok(env.new_string(replaced_accent_phrases_json)?.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsReplacePhonemeLength<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    accent_phrases_json: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let accent_phrases_json: String = env.get_string(&accent_phrases_json)?.into();
        let accent_phrases: Vec<voicevox_core::AccentPhraseModel> =
            serde_json::from_str(&accent_phrases_json).map_err(JavaApiError::DeJson)?;
        let style_id = style_id as u32;

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let replaced_accent_phrases = internal
            .replace_phoneme_length(&accent_phrases, voicevox_core::StyleId::new(style_id))?;

        let replaced_accent_phrases_json =
            serde_json::to_string(&replaced_accent_phrases).expect("should not fail");

        Ok(env.new_string(replaced_accent_phrases_json)?.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsReplaceMoraPitch<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    accent_phrases_json: JString<'local>,
    style_id: jint,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let accent_phrases_json: String = env.get_string(&accent_phrases_json)?.into();
        let accent_phrases: Vec<voicevox_core::AccentPhraseModel> =
            serde_json::from_str(&accent_phrases_json).map_err(JavaApiError::DeJson)?;
        let style_id = style_id as u32;

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let replaced_accent_phrases =
            internal.replace_mora_pitch(&accent_phrases, voicevox_core::StyleId::new(style_id))?;

        let replaced_accent_phrases_json =
            serde_json::to_string(&replaced_accent_phrases).expect("should not fail");

        Ok(env.new_string(replaced_accent_phrases_json)?.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsSynthesis<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    query_json: JString<'local>,
    style_id: jint,
    enable_interrogative_upspeak: jboolean,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let audio_query: String = env.get_string(&query_json)?.into();
        let audio_query: voicevox_core::AudioQueryModel =
            serde_json::from_str(&audio_query).map_err(JavaApiError::DeJson)?;
        let style_id = style_id as u32;

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let wave = {
            let options = voicevox_core::SynthesisOptions {
                enable_interrogative_upspeak: enable_interrogative_upspeak != 0,
                // ..Default::default()
            };
            internal.synthesis(
                &audio_query,
                voicevox_core::StyleId::new(style_id),
                &options,
            )?
        };

        let j_bytes = env.byte_array_from_slice(&wave)?;

        Ok(j_bytes.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsSynthesisMorphing<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    audio_query: JString<'local>,
    base_style_id: jint,
    target_style_id: jint,
    morph_rate: jdouble,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let audio_query = &env.get_string(&audio_query)?;
        let audio_query = &Cow::<str>::from(audio_query);
        let audio_query = &serde_json::from_str::<voicevox_core::AudioQueryModel>(audio_query)
            .map_err(JavaApiError::DeJson)?;

        let base_style_id = voicevox_core::StyleId::new(base_style_id as _);
        let target_style_id = voicevox_core::StyleId::new(target_style_id as _);

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let wav = &internal.synthesis_morphing(
            audio_query,
            base_style_id,
            target_style_id,
            morph_rate,
        )?;
        let wav = env.byte_array_from_slice(wav)?;
        Ok(wav.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsTtsFromKana<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    kana: JString<'local>,
    style_id: jint,
    enable_interrogative_upspeak: jboolean,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let kana: String = env.get_string(&kana)?.into();
        let style_id = style_id as u32;

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let wave = {
            let options = voicevox_core::TtsOptions {
                enable_interrogative_upspeak: enable_interrogative_upspeak != 0,
                // ..Default::default()
            };
            internal.tts_from_kana(&kana, voicevox_core::StyleId::new(style_id), &options)?
        };

        let j_bytes = env.byte_array_from_slice(&wave)?;

        Ok(j_bytes.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsTts<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    query_json: JString<'local>,
    style_id: jint,
    enable_interrogative_upspeak: jboolean,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let text: String = env.get_string(&query_json)?.into();
        let style_id = style_id as u32;

        let internal = env
            .get_rust_field::<_, _, Arc<voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>>>(
                &this, "handle",
            )?
            .clone();

        let wave = {
            let options = voicevox_core::TtsOptions {
                enable_interrogative_upspeak: enable_interrogative_upspeak != 0,
                // ..Default::default()
            };
            internal.tts(&text, voicevox_core::StyleId::new(style_id), &options)?
        };

        let j_bytes = env.byte_array_from_slice(&wave)?;

        Ok(j_bytes.into_raw())
    })
}

#[no_mangle]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Synthesizer_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        env.take_rust_field(&this, "handle")?;
        Ok(())
    })
}
