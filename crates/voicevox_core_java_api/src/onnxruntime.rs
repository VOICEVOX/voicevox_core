use std::ptr;

use duplicate::duplicate_item;
use jni::{
    objects::{JObject, JString},
    sys::jobject,
    JNIEnv,
};

use crate::common::throw_if_err;

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[duplicate_item(
    f CONST;
    [ Java_jp_hiroshiba_voicevoxcore_blocking_Onnxruntime_rsLibName ] [ LIB_NAME ];
    [ Java_jp_hiroshiba_voicevoxcore_blocking_Onnxruntime_rsLibVersion ] [ LIB_VERSION ];
    [ Java_jp_hiroshiba_voicevoxcore_blocking_Onnxruntime_rsLibVersionedFilename ] [ LIB_VERSIONED_FILENAME ];
    [ Java_jp_hiroshiba_voicevoxcore_blocking_Onnxruntime_rsLibUnversionedFilename ] [ LIB_UNVERSIONED_FILENAME ];
)]
#[unsafe(no_mangle)]
extern "system" fn f(env: JNIEnv<'_>) -> jobject {
    throw_if_err(env, ptr::null_mut(), |env| {
        let s = env.new_string(voicevox_core::blocking::Onnxruntime::CONST)?;
        Ok(s.into_raw())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Onnxruntime_rsNew<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    filename: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let filename = String::from(env.get_string(&filename)?);
        let internal = voicevox_core::blocking::Onnxruntime::load_once()
            .filename(filename)
            .exec()?;
        env.set_rust_field(&this, "handle", internal)?;
        Ok(())
    })
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
unsafe extern "system" fn Java_jp_hiroshiba_voicevoxcore_blocking_Onnxruntime_rsSupportedDevices<
    'local,
>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, ptr::null_mut(), |env| {
        let this = *env.get_rust_field::<_, _, &'static voicevox_core::blocking::Onnxruntime>(
            &this, "handle",
        )?;
        let json = this.supported_devices()?.to_json().to_string();
        let json = env.new_string(json)?;
        Ok(json.into_raw())
    })
}
