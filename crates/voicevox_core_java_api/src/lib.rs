use jni::{
    objects::{JClass, JString},
    sys::{jboolean, jstring},
    JNIEnv,
};

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_test<'local>(
    mut env: JNIEnv<'local>,
    class: JClass<'local>,
) -> jboolean {
    true.into()
}
