use jni::{
    JNIEnv,
    objects::JClass,
    sys::{jdouble, jint},
};

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_Frames_rsFrameRate<'local>(
    _: JNIEnv<'local>,
    _: JClass<'local>,
) -> jdouble {
    voicevox_core::FRAME_RATE
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_Frames_rsWaveSamplesPerFrame<'local>(
    _: JNIEnv<'local>,
    _: JClass<'local>,
) -> jint {
    voicevox_core::WAVE_SAMPLES_PER_FRAME.into()
}
