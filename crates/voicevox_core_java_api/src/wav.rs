use jni::{
    JNIEnv,
    objects::{JByteArray, JClass},
    sys::{jboolean, jint, jobject},
};
use voicevox_core::wav_from_s16le;

use crate::common::throw_if_err;

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_Wav_rsWavFromS16le<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'_>,
    pcm: JByteArray<'local>,
    sample_rate: jint,
    stereo: jboolean,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let pcm = env.convert_byte_array(&pcm)?;
        let wav = wav_from_s16le(&pcm, sample_rate as u32, stereo != 0);
        Ok(env.byte_array_from_slice(&wav)?.into_raw())
    })
}
