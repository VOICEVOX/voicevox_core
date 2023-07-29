mod utils;
use jni::{
    objects::{JClass, JObject, JString, JValueGen},
    sys::{jboolean, jlong},
    JNIEnv,
};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    if cfg!(target_os = "android") {
        android_logger::init_once(
            android_logger::Config::default()
                .with_tag("VoicevoxCore")
                .with_filter(
                    android_logger::FilterBuilder::new()
                        .parse(
                            "error,voicevox_core=info,voicevox_core_java_api=info,onnxruntime=info",
                        )
                        .build(),
                ),
        );
    } else {
        // TODO: Android以外でのログ出力を良い感じにする。（System.Loggerを使う？）
        use chrono::SecondsFormat;
        use std::{
            env, fmt,
            io::{self, IsTerminal, Write},
        };
        use tracing_subscriber::{fmt::format::Writer, EnvFilter};

        let _ = tracing_subscriber::fmt()
            .with_env_filter(if env::var_os(EnvFilter::DEFAULT_ENV).is_some() {
                EnvFilter::from_default_env()
            } else {
                "error,voicevox_core=info,voicevox_core_c_api=info,onnxruntime=info".into()
            })
            .with_timer(local_time as fn(&mut Writer<'_>) -> _)
            .with_ansi(out().is_terminal() && env_allows_ansi())
            .with_writer(out)
            .try_init();

        fn local_time(wtr: &mut Writer<'_>) -> fmt::Result {
            // ローカル時刻で表示はするが、そのフォーマットはtracing-subscriber本来のものに近いようにする。
            // https://github.com/tokio-rs/tracing/blob/tracing-subscriber-0.3.16/tracing-subscriber/src/fmt/time/datetime.rs#L235-L241
            wtr.write_str(&chrono::Local::now().to_rfc3339_opts(SecondsFormat::Micros, false))
        }

        fn out() -> impl IsTerminal + Write {
            io::stderr()
        }

        fn env_allows_ansi() -> bool {
            // https://docs.rs/termcolor/1.2.0/src/termcolor/lib.rs.html#245-291
            // ただしWindowsではPowerShellっぽかったらそのまま許可する。
            // ちゃんとやるなら`ENABLE_VIRTUAL_TERMINAL_PROCESSING`をチェックするなり、そもそも
            // fwdansiとかでWin32の色に変換するべきだが、面倒。
            env::var_os("TERM").map_or(
                cfg!(windows) && env::var_os("PSModulePath").is_some(),
                |term| term != "dumb",
            ) && env::var_os("NO_COLOR").is_none()
        }
    }
    Runtime::new().unwrap()
});

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_rsNewWithoutDic<'local>(
    mut env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jboolean {
    let internal = voicevox_core::OpenJtalk::new_without_dic();
    let internal_ptr = Box::into_raw(Box::new(internal));

    unwrap_with_throw!(
        env,
        env.set_field(
            this,
            "internalPtr",
            "J",
            JValueGen::Long(internal_ptr as jlong),
        )
    );

    true as jboolean
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_rsNewWithInitialize<'local>(
    mut env: JNIEnv<'local>,
    this: JObject<'local>,
    open_jtalk_dict_dir: JString<'local>,
) -> jboolean {
    let open_jtalk_dict_dir = env
        .get_string(&open_jtalk_dict_dir)
        .expect("invalid java string");
    let open_jtalk_dict_dir = open_jtalk_dict_dir.to_str().unwrap();

    let internal = unwrap_with_throw!(
        env,
        voicevox_core::OpenJtalk::new_with_initialize(open_jtalk_dict_dir)
    );
    let internal_ptr = Box::into_raw(Box::new(internal));

    unwrap_with_throw!(
        env,
        env.set_field(
            this,
            "internalPtr",
            "J",
            JValueGen::Long(internal_ptr as jlong),
        )
    );

    true as jboolean
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_VoiceModel_rsFromPath<'local>(
    mut env: JNIEnv<'local>,
    this: JClass<'local>,
    model_path: JString<'local>,
) -> jboolean {
    let model_path = env.get_string(&model_path).expect("invalid java string");
    let model_path = model_path.to_str().unwrap();

    let internal = unwrap_with_throw!(
        env,
        RUNTIME.block_on(voicevox_core::VoiceModel::from_path(model_path))
    );
    let id = unwrap_with_throw!(env, env.new_string(internal.id().raw_voice_model_id()));
    let internal_ptr = Box::into_raw(Box::new(internal));

    unwrap_with_throw!(
        env,
        env.set_field(
            &this,
            "internalPtr",
            "J",
            JValueGen::Long(internal_ptr as jlong),
        )
    );

    unwrap_with_throw!(
        env,
        env.set_field(&this, "id", "Ljava/lang/String;", JValueGen::Object(&id))
    );
    true as jboolean
}
