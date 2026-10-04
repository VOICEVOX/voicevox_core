use jni::{JNIEnv, objects::JClass, sys::jdouble};

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_Frames_rsFrameRate<'local>(
    _: JNIEnv<'local>,
    _: JClass<'local>,
) -> jdouble {
    voicevox_core::FRAME_RATE
}
