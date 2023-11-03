use crate::common::throw_if_err;
use jni::{sys::jobject, JNIEnv};
#[no_mangle]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_VoicevoxCoreInfo_rsGetVersion(
    env: JNIEnv<'_>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let version = env.new_string(env!("CARGO_PKG_VERSION"))?;
        Ok(version.into_raw())
    })
}
#[no_mangle]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_VoicevoxCoreInfo_rsGetSupportedDevicesJson(
    env: JNIEnv<'_>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let supported_devices = voicevox_core::SupportedDevices::create()?;
        let json = serde_json::to_string(&supported_devices).expect("Should not fail");
        let json = env.new_string(json)?;
        Ok(json.into_raw())
    })
}
