use std::ptr;

use duplicate::duplicate_item;
use jni::{
    JNIEnv,
    objects::{JObject, JString},
    sys::jobject,
};
use voicevox_core::__internal::interop::ToJsonValue as _;

use crate::{common::throw_if_err, object};

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
            .perform()?;
        // SAFETY:
        // - The safety contract must be upheld by the caller.
        // - `jp.hiroshiba.voicevoxcore.blocking.Onnxruntime.handle` must correspond to
        //   `&'static voicevox_core::blocking::Onnxruntime`.
        unsafe { env.set_rust_field(&this, "handle", internal) }?;
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
        let this = *unsafe {
            // SAFETY:
            // - The safety contract must be upheld by the caller.
            // - `jp.hiroshiba.voicevoxcore.blocking.Onnxruntime.handle` must correspond to
            //   `&'static voicevox_core::blocking::Onnxruntime`.
            env.get_rust_field::<_, _, &'static voicevox_core::blocking::Onnxruntime>(
                &this, "handle",
            )
        }?;
        let devices = this.supported_devices()?;

        assert!(match devices.to_json_value() {
            serde_json::Value::Object(o) => o.len() == 3, // `cpu`, `cuda`, `dml`
            _ => false,
        });

        let devices = env.new_object(
            object!("SupportedDevices"),
            "(ZZZ)V",
            &[devices.cpu.into(), devices.cuda.into(), devices.dml.into()],
        )?;
        Ok(devices.into_raw())
    })
}
