use std::ptr;

use duplicate::duplicate_item;
use jni::{
    JNIEnv,
    objects::{JObject, JString},
    sys::{jint, jobject},
};
use voicevox_core::__internal::interop::ToJsonValue as _;

use crate::{common::throw_if_err, object};

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[duplicate_item(
    f CONST;
    [ Java_jp_hiroshiba_voicevoxcore_blocking_Onnxruntime_rsLibMinRequiredMinorVersion ] [ LIB_MIN_REQUIRED_MINOR_VERSION ];
    [ Java_jp_hiroshiba_voicevoxcore_blocking_Onnxruntime_rsLibMaxSupportedMinorVersion ] [ LIB_MAX_SUPPORTED_MINOR_VERSION ];
)]
#[unsafe(no_mangle)]
extern "system" fn f(_: JNIEnv<'_>) -> jint {
    voicevox_core::blocking::Onnxruntime::CONST
        .try_into()
        .expect("ONNX Runtime minor version is not considered to be so large")
}

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[duplicate_item(
    f CONST;
    [ Java_jp_hiroshiba_voicevoxcore_blocking_Onnxruntime_rsLibRecommendedName ] [ LIB_RECOMMENDED_NAME ];
    [ Java_jp_hiroshiba_voicevoxcore_blocking_Onnxruntime_rsLibRecommendedVersion ] [ LIB_RECOMMENDED_VERSION ];
    [ Java_jp_hiroshiba_voicevoxcore_blocking_Onnxruntime_rsLibRecommendedVersionedFilename ] [ LIB_RECOMMENDED_VERSIONED_FILENAME ];
    [ Java_jp_hiroshiba_voicevoxcore_blocking_Onnxruntime_rsLibRecommendedUnversionedFilename ] [ LIB_RECOMMENDED_UNVERSIONED_FILENAME ];
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
            serde_json::Value::Object(o) => o.len() == 4, // `cpu`, `cuda`, `dml`, `webgpu`
            _ => false,
        });

        let devices = env.new_object(
            object!("SupportedDevices"),
            "(ZZZZ)V",
            &[
                devices.cpu.into(),
                devices.cuda.into(),
                devices.dml.into(),
                devices.webgpu.into(),
            ],
        )?;
        Ok(devices.into_raw())
    })
}
