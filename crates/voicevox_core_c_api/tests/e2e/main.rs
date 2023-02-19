use std::ffi::{c_char, c_int};

use assert_cmd::assert::AssertResult;
use easy_ext::ext;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};

use self::assert_cdylib::Utf8Output;

mod assert_cdylib;
mod float_assert;
mod mask;
mod snapshots;
mod testcases;

fn main() -> anyhow::Result<()> {
    return assert_cdylib::exec::<TestSuite>();

    enum TestSuite {}

    impl assert_cdylib::TestSuite for TestSuite {
        type TestCase = TestCase;

        const TARGET_DIR: &'static str = "../../target";

        const CDYLIB_NAME: &'static str = "voicevox_core";

        const BUILD_ENVS: &'static [(&'static str, &'static str)] = &[
            // 他の単体テストが動いているときにonnxruntime-sysの初回ビルドを行うと、Windows環境だと
            // `$ORT_OUT_DIR`のハックが問題を起こす。そのためこのハック自体を無効化する
            //
            // featuresの差分を出さないように`cargo build`することができればonnxruntime-sysの
            // ビルド自体がされないのだが、このバイナリから`cargo build`の状況を知るのは無理に近い
            ("ORT_OUT_DIR", ""),
            // DirectMLとCUDAは無効化
            ("ORT_USE_CUDA", "0"),
        ];

        const RUNTIME_ENVS: &'static [(&'static str, &'static str)] =
            &[("VV_MODELS_ROOT_DIR", "../../model")];

        const TESTCASES: &'static [Self::TestCase] = &[TestCase::CompatibleEngine];

        unsafe fn exec(testcase: Self::TestCase, lib: &Library) -> anyhow::Result<()> {
            use self::testcases::*;

            let symbols = Symbols::new(lib)?;

            match testcase {
                TestCase::CompatibleEngine => compatible_engine::exec(symbols),
            }
        }

        fn assert_output(testcase: Self::TestCase, output: Utf8Output) -> AssertResult {
            use self::testcases::*;

            match testcase {
                TestCase::CompatibleEngine => compatible_engine::assert_output(output),
            }
        }
    }

    #[derive(Clone, Copy, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum TestCase {
        CompatibleEngine,
    }
}

struct Symbols<'lib> {
    initialize: Symbol<'lib, unsafe extern "C" fn(bool, c_int, bool) -> bool>,
    load_model: Symbol<'lib, unsafe extern "C" fn(i64) -> bool>,
    is_model_loaded: Symbol<'lib, unsafe extern "C" fn(i64) -> bool>,
    finalize: Symbol<'lib, unsafe extern "C" fn()>,
    metas: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    supported_devices: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    yukarin_s_forward:
        Symbol<'lib, unsafe extern "C" fn(i64, *mut i64, *mut i64, *mut f32) -> bool>,
    yukarin_sa_forward: Symbol<
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
    decode_forward: Symbol<
        'lib,
        unsafe extern "C" fn(i64, i64, *mut f32, *mut f32, *mut i64, *mut f32) -> bool,
    >,
}

#[ext]
impl<'lib> Symbols<'lib> {
    unsafe fn new(lib: &'lib Library) -> Result<Self, libloading::Error>
    where
        Self: Sized,
    {
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
            supported_devices,
            yukarin_s_forward,
            yukarin_sa_forward,
            decode_forward,
        ))
    }
}
