use std::sync::Arc;

use jni::{
    JNIEnv,
    objects::JObject,
    sys::{jdouble, jlong},
};
use voicevox_core::AudioFeature;

use crate::common::throw_if_err;

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_AudioFeature_rsFrameRate(
    _: JNIEnv<'_>,
) -> jdouble {
    AudioFeature::FRAME_RATE
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_AudioFeature_rsGetFrameLength<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jlong {
    // call the getter AudioFeature::frame_length
    throw_if_err(env, 0, |env| {
        let internal = unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.AudioFeature.handle` must correspond to
            //   `Arc<voicevox_core::AudioFeature>`.
            type RustField = Arc<AudioFeature>;
            env.get_rust_field::<_, _, RustField>(&this, "handle")
        }?;

        Ok(internal.frame_length())
    })
    .try_into()
    .expect("`frame_length` is not considered to be so large")
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_AudioFeature_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.AudioFeature.handle` must correspond to
            //   `Arc<voicevox_core::AudioFeature>`.
            type RustField = Arc<AudioFeature>;
            env.take_rust_field::<_, _, RustField>(&this, "handle")
        }?;
        Ok(())
    })
}
