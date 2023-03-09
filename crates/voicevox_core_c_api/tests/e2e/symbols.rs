use std::ffi::{c_char, c_int};

use libloading::{Library, Symbol};

/// voicevox\_core\_c\_apiのcdylibのシンボルを集めたもの。
pub(crate) struct Symbols<'lib> {
    pub(crate) initialize: Symbol<'lib, unsafe extern "C" fn(bool, c_int, bool) -> bool>,
    pub(crate) load_model: Symbol<'lib, unsafe extern "C" fn(i64) -> bool>,
    pub(crate) is_model_loaded: Symbol<'lib, unsafe extern "C" fn(i64) -> bool>,
    pub(crate) finalize: Symbol<'lib, unsafe extern "C" fn()>,
    pub(crate) metas: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    pub(crate) last_error_message: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    pub(crate) supported_devices: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    pub(crate) yukarin_s_forward:
        Symbol<'lib, unsafe extern "C" fn(i64, *mut i64, *mut i64, *mut f32) -> bool>,
    pub(crate) yukarin_sa_forward: Symbol<
        'lib,
        unsafe extern "C" fn(
            i64,
            *mut i64,
            *mut i64,
            *mut i64,
            *mut i64,
            *mut i64,
            *mut i64,
            *mut i64,
            *mut f32,
        ) -> bool,
    >,
    pub(crate) decode_forward: Symbol<
        'lib,
        unsafe extern "C" fn(i64, i64, *mut f32, *mut f32, *mut i64, *mut f32) -> bool,
    >,
}

impl<'lib> Symbols<'lib> {
    pub(crate) unsafe fn new(lib: &'lib Library) -> Result<Self, libloading::Error> {
        macro_rules! new(($($name:ident),* $(,)?) => {
            Self {
                $(
                    $name: lib.get(stringify!($name).as_ref())?,
                )*
            }
        });

        Ok(new!(
            initialize,
            load_model,
            is_model_loaded,
            finalize,
            metas,
            last_error_message,
            supported_devices,
            yukarin_s_forward,
            yukarin_sa_forward,
            decode_forward,
        ))
    }
}
