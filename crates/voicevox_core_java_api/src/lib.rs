mod utils;
use crate::utils::throw_if_err;
use jni::{
    objects::{JObject, JString, JValueGen},
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
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jboolean {
    throw_if_err(env, false, |env| {
        let internal = voicevox_core::OpenJtalk::new_without_dic();
        let internal_ptr = Box::into_raw(Box::new(internal));

        env.set_field(
            this,
            "internalPtr",
            "J",
            JValueGen::Long(internal_ptr as jlong),
        )?;
        Ok(true)
    }) as jboolean
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_OpenJtalk_rsNewWithInitialize<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    open_jtalk_dict_dir: JString<'local>,
) -> jboolean {
    throw_if_err(env, false, |env| {
        let open_jtalk_dict_dir = env.get_string(&open_jtalk_dict_dir)?;
        let open_jtalk_dict_dir = open_jtalk_dict_dir.to_str()?;

        let internal = voicevox_core::OpenJtalk::new_with_initialize(open_jtalk_dict_dir)?;
        let internal_ptr = Box::into_raw(Box::new(internal));

        env.set_field(
            this,
            "internalPtr",
            "J",
            JValueGen::Long(internal_ptr as jlong),
        )?;
        Ok(true)
    }) as jboolean
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_VoiceModel_rsFromPath<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model_path: JString<'local>,
) -> jboolean {
    throw_if_err(env, false, |env| {
        let model_path = env.get_string(&model_path)?;
        let model_path = model_path.to_str()?;

        let internal = RUNTIME.block_on(voicevox_core::VoiceModel::from_path(model_path))?;
        let internal_ptr = Box::into_raw(Box::new(&internal));

        env.set_field(
            &this,
            "internalPtr",
            "J",
            JValueGen::Long(internal_ptr as jlong),
        )?;

        env.set_field(
            &this,
            "id",
            "Ljava/lang/String;",
            JValueGen::Object(&env.new_string(internal.id().raw_voice_model_id())?.into()),
        )?;
        let speakers = env.new_object_array(
            internal.metas().len() as i32,
            object!("SpeakerMeta"),
            JObject::null(),
        )?;
        for (i, meta) in internal.metas().iter().enumerate() {
            let j_meta = env.new_object(object!("SpeakerMeta"), "()V", &[])?;
            env.set_field(
                &j_meta,
                "name",
                "Ljava/lang/String;",
                JValueGen::Object(&env.new_string(meta.name())?.into()),
            )?;
            let j_styles = env.new_object_array(
                meta.styles().len() as i32,
                object!("StyleMeta"),
                JObject::null(),
            )?;
            for (j, style) in meta.styles().iter().enumerate() {
                let j_style = env.new_object(object!("StyleMeta"), "()V", &[])?;
                env.set_field(
                    &j_style,
                    "name",
                    "Ljava/lang/String;",
                    JValueGen::Object(&env.new_string(style.name())?.into()),
                )?;
                env.set_field(
                    &j_style,
                    "id",
                    "I",
                    JValueGen::Int(style.id().raw_id() as i32),
                )?;
                env.set_object_array_element(&j_styles, j as i32, j_style)?;
            }
            env.set_field(
                &j_meta,
                "styles",
                concat!("[", object!("StyleMeta")),
                JValueGen::Object(&j_styles),
            )?;
            env.set_field(
                &j_meta,
                "speakerUuid",
                "Ljava/lang/String;",
                JValueGen::Object(&env.new_string(meta.speaker_uuid())?.into()),
            )?;
            env.set_field(
                &j_meta,
                "version",
                "Ljava/lang/String;",
                JValueGen::Object(&env.new_string(meta.version().raw_version())?.into()),
            )?;

            env.set_object_array_element(&speakers, i as i32, j_meta)?;
        }
        Ok(true)
    }) as jboolean
}
