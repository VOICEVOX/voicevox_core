use std::{error::Error as _, iter};

use derive_more::From;
use jni::{objects::JThrowable, JNIEnv};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    if cfg!(target_os = "android") {
        android_logger::init_once(
            android_logger::Config::default()
                .with_tag("VoicevoxCore")
                .with_filter(
                android_logger::FilterBuilder::new()
                    .parse("error,voicevox_core=info,voicevox_core_java_api=info,onnxruntime=error")
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
                "error,voicevox_core=info,voicevox_core_c_api=info,onnxruntime=error".into()
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

#[macro_export]
macro_rules! object {
    ($name: literal) => {
        concat!("jp/hiroshiba/voicevoxcore/", $name)
    };
}
#[macro_export]
macro_rules! object_type {
    ($name: literal) => {
        concat!("Ljp/hiroshiba/voicevoxcore/", $name, ";")
    };
}
#[macro_export]
macro_rules! enum_object {
    ($env: ident, $name: literal, $variant: literal) => {
        $env.get_static_field(object!($name), $variant, object_type!($name))
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to get field {}",
                    concat!($variant, "L", object!($name), ";")
                )
            })
            .l()
    };
}

pub fn throw_if_err<T, F>(mut env: JNIEnv<'_>, fallback: T, inner: F) -> T
where
    F: FnOnce(&mut JNIEnv<'_>) -> Result<T, JavaApiError>,
{
    match inner(&mut env) {
        Ok(value) => value as _,
        Err(error) => {
            // Java側の例外は無視する。
            // env.exception_clear()してもいいが、errorのメッセージは"Java exception was thrown"
            // となり、デバッグが困難になるため、そのままにしておく。
            if !env.exception_check().unwrap_or(false) {
                macro_rules! or_panic {
                    ($result:expr) => {
                        $result.unwrap_or_else(|_| {
                            panic!("Failed to throw exception, original error: {error:?}")
                        })
                    };
                }

                match &error {
                    JavaApiError::RustApi(error) => {
                        macro_rules! class {
                            ($($variant:ident),* $(,)?) => {
                                match error.kind() {
                                    $(
                                        voicevox_core::ErrorKind::$variant => concat!(
                                            "jp/hiroshiba/voicevoxcore/exceptions/",
                                            stringify!($variant),
                                            "Exception",
                                        ),
                                    )*
                                }
                            };
                        }

                        let class = class!(
                            LoadOpenjtalkSystemDic,
                            GpuSupport,
                            OpenZipFile,
                            ReadZipEntry,
                            ModelAlreadyLoaded,
                            StyleAlreadyLoaded,
                            InvalidModelData,
                            GetSupportedDevices,
                            StyleNotFound,
                            ModelNotFound,
                            InferenceFailed,
                            ExtractFullContextLabel,
                            ParseKana,
                            LoadUserDict,
                            SaveUserDict,
                            WordNotFound,
                            UseUserDict,
                            InvalidWord,
                        );

                        let mut sources =
                            iter::successors(error.source(), |&source| source.source())
                                .collect::<Vec<_>>()
                                .into_iter()
                                .rev();

                        // FIXME: `.unwrap()`ではなく、ちゃんと`.expect()`とかを書く

                        let exc = JThrowable::from(if let Some(innermost) = sources.next() {
                            let innermost = env
                                .new_object(
                                    "java/lang/RuntimeException",
                                    "(Ljava/lang/String;)V",
                                    &[(&env.new_string(innermost.to_string()).unwrap()).into()],
                                )
                                .unwrap();

                            let cause = sources.fold(innermost, |cause, source| {
                                env.new_object(
                                    "java/lang/RuntimeException",
                                    "(Ljava/lang/String;Ljava/lang/Throwable;)V",
                                    &[
                                        (&env.new_string(source.to_string()).unwrap()).into(),
                                        (&cause).into(),
                                    ],
                                )
                                .unwrap()
                            });

                            env.new_object(
                                class,
                                "(Ljava/lang/String;Ljava/lang/Throwable;)V",
                                &[
                                    (&env.new_string(error.to_string()).unwrap()).into(),
                                    (&cause).into(),
                                ],
                            )
                            .unwrap()
                        } else {
                            env.new_object(
                                class,
                                "(Ljava/lang/String;)V",
                                &[(&env.new_string(error.to_string()).unwrap()).into()],
                            )
                            .unwrap()
                        });

                        or_panic!(env.throw(exc));
                    }
                    JavaApiError::Jni(error) => {
                        or_panic!(env.throw_new("java/lang/RuntimeException", error.to_string()))
                    }
                    JavaApiError::Uuid(error) => {
                        or_panic!(
                            env.throw_new("java/lang/IllegalArgumentException", error.to_string())
                        )
                    }
                    JavaApiError::DeJson(error) => {
                        or_panic!(
                            env.throw_new("java/lang/IllegalArgumentException", error.to_string())
                        )
                    }
                };
            }
            fallback
        }
    }
}

#[derive(From, Debug)]
pub enum JavaApiError {
    #[from]
    RustApi(voicevox_core::Error),

    #[from]
    Jni(jni::errors::Error),

    #[from]
    Uuid(uuid::Error),

    DeJson(serde_json::Error),
}
