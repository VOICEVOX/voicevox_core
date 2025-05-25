use jni::{JNIEnv, objects::JObject};

// SAFETY: voicevox_core_java_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
extern "system" fn Java_jp_hiroshiba_voicevoxcore_internal_Dll_00024LoggerInitializer_initLogger(
    _: JNIEnv<'_>,
    _: JObject<'_>,
) {
    if cfg!(target_os = "android") {
        android_logger::init_once(
            android_logger::Config::default()
                .with_tag("VoicevoxCore")
                .with_filter(
                    android_logger::FilterBuilder::new()
                        // FIXME: ortも`warn`は出すべき
                        .parse("error,voicevox_core=info,voicevox_core_java_api=info,ort=error")
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
        use tracing_subscriber::{EnvFilter, fmt::format::Writer};

        tracing_subscriber::fmt()
            .with_env_filter(if env::var_os(EnvFilter::DEFAULT_ENV).is_some() {
                EnvFilter::from_default_env()
            } else {
                // FIXME: `c_api`じゃないし、ortも`warn`は出すべき
                // FIXME: c_apiじゃなくてjava_api
                "error,voicevox_core=info,voicevox_core_c_api=info,ort=error".into()
            })
            .with_timer(local_time as fn(&mut Writer<'_>) -> _)
            .with_ansi(out().is_terminal() && env_allows_ansi())
            .with_writer(out)
            .init();

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
}
