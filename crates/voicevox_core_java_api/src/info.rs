use crate::common::throw_if_err;
use jni::{JNIEnv, sys::jobject};

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_GlobalInfo_rsGetVersion(
    env: JNIEnv<'_>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let version = env.new_string(env!("CARGO_PKG_VERSION"))?;
        Ok(version.into_raw())
    })
}
