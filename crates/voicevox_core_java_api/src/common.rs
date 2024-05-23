use std::{error::Error as _, iter};

use derive_more::From;
use easy_ext::ext;
use jni::{
    objects::{JObject, JThrowable},
    JNIEnv,
};
use uuid::Uuid;

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

pub(crate) fn throw_if_err<T, F>(mut env: JNIEnv<'_>, fallback: T, inner: F) -> T
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
                            NotLoadedOpenjtalkDict,
                            GpuSupport,
                            OpenZipFile,
                            ReadZipEntry,
                            InvalidModelFormat,
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
pub(crate) enum JavaApiError {
    #[from]
    RustApi(voicevox_core::Error),

    #[from]
    Jni(jni::errors::Error),

    #[from]
    Uuid(uuid::Error),

    DeJson(serde_json::Error),
}

#[ext(JNIEnvExt)]
pub(crate) impl JNIEnv<'_> {
    fn new_uuid(&mut self, uuid: Uuid) -> jni::errors::Result<JObject<'_>> {
        let uuid = uuid.as_u128();
        let msbs = i64::try_from(uuid >> 64).expect("128 - 64 = 64").into();
        let lsbs = (uuid as i64).into();
        self.new_object("java/util/UUID", "(JJ)V", &[msbs, lsbs])
    }

    fn get_uuid(&mut self, obj: &JObject<'_>) -> jni::errors::Result<Uuid> {
        let mut get_bits = |name| {
            let v = self.call_method(obj, name, "()J", &[])?.j()?;
            Ok::<_, jni::errors::Error>(u128::from(v as u64))
        };
        let msbs = get_bits("getMostSignificantBits")?;
        let lsbs = get_bits("getLeastSignificantBits")?;
        Ok(Uuid::from_u128((msbs << 64) + lsbs))
    }
}
