use std::{error::Error as _, iter, mem, ops::Deref};

use derive_more::From;
use easy_ext::ext;
use jni::{
    objects::{JObject, JThrowable},
    JNIEnv,
};
use tracing::{debug, warn};
use uuid::Uuid;
use voicevox_core::__internal::interop::raii::MaybeClosed;

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
                            InitInferenceRuntime,
                            OpenZipFile,
                            ReadZipEntry,
                            InvalidModelFormat,
                            ModelAlreadyLoaded,
                            StyleAlreadyLoaded,
                            InvalidModelData,
                            GetSupportedDevices,
                            StyleNotFound,
                            ModelNotFound,
                            RunModel,
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
                    JavaApiError::IllegalState(msg) => {
                        or_panic!(env.throw_new("java/lang/IllegalStateException", msg))
                    }
                };
            }
            fallback
        }
    }
}

type JavaApiResult<T> = Result<T, JavaApiError>;

#[derive(From, Debug)]
pub(crate) enum JavaApiError {
    #[from]
    RustApi(voicevox_core::Error),

    #[from]
    Jni(jni::errors::Error),

    #[from]
    Uuid(uuid::Error),

    DeJson(serde_json::Error),

    IllegalState(String),
}

pub(crate) struct Closable<T: HasJavaClassIdent>(std::sync::RwLock<MaybeClosed<T>>);

impl<T: HasJavaClassIdent + 'static> Closable<T> {
    pub(crate) fn new(content: T) -> Self {
        Self(MaybeClosed::Open(content).into())
    }

    pub(crate) fn read(&self) -> JavaApiResult<impl Deref<Target = T> + '_> {
        let lock = self.0.try_read().map_err(|e| match e {
            std::sync::TryLockError::Poisoned(e) => panic!("{e}"),
            std::sync::TryLockError::WouldBlock => {
                JavaApiError::IllegalState(format!("The `{}` is being closed", T::JAVA_CLASS_IDENT))
            }
        })?;

        voicevox_core::__internal::interop::raii::try_map_guard(lock, |lock| match &**lock {
            MaybeClosed::Open(content) => Ok(content),
            MaybeClosed::Closed => Err(JavaApiError::IllegalState(format!(
                "The `{}` is closed",
                T::JAVA_CLASS_IDENT,
            ))),
        })
    }

    pub(crate) fn close(&self) {
        let lock = &mut *match self.0.try_write() {
            Ok(lock) => lock,
            Err(std::sync::TryLockError::Poisoned(e)) => panic!("{e}"),
            Err(std::sync::TryLockError::WouldBlock) => {
                self.0.write().unwrap_or_else(|e| panic!("{e}"))
            }
        };

        if matches!(*lock, MaybeClosed::Open(_)) {
            debug!("Closing a `{}`", T::JAVA_CLASS_IDENT);
        }
        drop(mem::replace(lock, MaybeClosed::Closed));
    }
}

impl<T: HasJavaClassIdent> Drop for Closable<T> {
    fn drop(&mut self) {
        let content = mem::replace(
            &mut *self.0.write().unwrap_or_else(|e| panic!("{e}")),
            MaybeClosed::Closed,
        );
        if let MaybeClosed::Open(content) = content {
            warn!(
                "デストラクタにより`{}`のクローズを行います。通常は、可能な限り`close`でクローズす\
                 るようにして下さい",
                T::JAVA_CLASS_IDENT,
            );
            drop(content);
        }
    }
}

pub(crate) trait HasJavaClassIdent {
    const JAVA_CLASS_IDENT: &str;
}

#[ext(JNIEnvExt)]
pub(crate) impl JNIEnv<'_> {
    fn new_uuid(&mut self, uuid: Uuid) -> jni::errors::Result<JObject<'_>> {
        let (msbs, lsbs) = split_uuid(uuid);
        self.new_object("java/util/UUID", "(JJ)V", &[msbs.into(), lsbs.into()])
    }

    fn get_uuid(&mut self, obj: &JObject<'_>) -> jni::errors::Result<Uuid> {
        let mut get_bits = |method_name| self.call_method(obj, method_name, "()J", &[])?.j();
        let msbs = get_bits("getMostSignificantBits")?;
        let lsbs = get_bits("getLeastSignificantBits")?;
        Ok(construct_uuid(msbs, lsbs))
    }
}

fn split_uuid(uuid: Uuid) -> (i64, i64) {
    let uuid = uuid.as_u128();
    let msbs = (uuid >> 64) as _;
    let lsbs = uuid as _;
    (msbs, lsbs)
}

fn construct_uuid(msbs: i64, lsbs: i64) -> Uuid {
    return Uuid::from_u128((to_u128(msbs) << 64) + to_u128(lsbs));

    fn to_u128(bits: i64) -> u128 {
        (bits as u64).into()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use uuid::{uuid, Uuid};

    #[rstest]
    #[case(uuid!("a1a2a3a4-b1b2-c1c2-d1d2-e1e2e3e4e5e6"))]
    #[case(uuid!("00000000-0000-0000-0000-000000000000"))]
    #[case(uuid!("00000000-0000-0000-ffff-ffffffffffff"))]
    #[case(uuid!("ffffffff-ffff-ffff-0000-000000000000"))]
    #[case(uuid!("ffffffff-ffff-ffff-ffff-ffffffffffff"))]
    fn uuid_conversion_works(#[case] uuid: Uuid) {
        let (msbs, lsbs) = super::split_uuid(uuid);
        assert_eq!(uuid, super::construct_uuid(msbs, lsbs));
    }
}
