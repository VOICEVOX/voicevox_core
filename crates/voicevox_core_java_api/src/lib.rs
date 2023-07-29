mod utils;
use jni::{
    objects::{JClass, JString},
    sys::jlong,
    JNIEnv,
};

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_rsNewWithoutDic<'local>(
    mut _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jlong {
    let internal = voicevox_core::OpenJtalk::new_without_dic();
    let internal_ptr = Box::into_raw(Box::new(internal));

    internal_ptr as jlong
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_rsNewWithInitialize<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    open_jtalk_dict_dir: JString<'local>,
) -> jlong {
    let open_jtalk_dict_dir = env
        .get_string(&open_jtalk_dict_dir)
        .expect("invalid java string");
    let open_jtalk_dict_dir = open_jtalk_dict_dir.to_str().unwrap();

    let internal = unwrap_with_throw!(
        env,
        voicevox_core::OpenJtalk::new_with_initialize(open_jtalk_dict_dir)
    );
    let internal_ptr = Box::into_raw(Box::new(internal));

    internal_ptr as jlong
}
