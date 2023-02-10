use std::{
    ffi::{c_char, c_int},
    path::Path,
    process::Output,
};

use assert_cmd::assert::AssertResult;
use clap::{Parser as _, ValueEnum};
use duct::cmd;
use heck::ToSnakeCase as _;
use libloading::{Library, Symbol};
use libtest_mimic::Trial;
use strum::IntoStaticStr;

mod operations;

fn main() -> anyhow::Result<()> {
    if let Ok(ExecVoicevoxCApiE2eTest {
        exec_voicevox_c_api_e2e_test,
    }) = ExecVoicevoxCApiE2eTest::try_parse()
    {
        return exec_voicevox_c_api_e2e_test.exec();
    }

    let args = &libtest_mimic::Arguments::parse();

    if args.ignored || args.include_ignored {
        cmd!(env!("CARGO"), "build", "--lib")
            .env("ORT_OUT_DIR", "")
            .run()?;
    }

    let tests = Test::value_variants()
        .iter()
        .copied()
        .map(Into::into)
        .collect();

    libtest_mimic::run(args, tests).exit();
}

#[derive(clap::Parser)]
struct ExecVoicevoxCApiE2eTest {
    #[arg(long, required(true))]
    exec_voicevox_c_api_e2e_test: Test,
}

#[derive(Clone, Copy, ValueEnum, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
enum Test {
    CompatibleEngine,
    VoicevoxGetVersion,
}

impl Test {
    fn exec(self) -> anyhow::Result<()> {
        use operations::*;

        let cdylib_path = Path::new("..")
            .join("..")
            .join("target")
            .join("debug")
            .join(libloading::library_filename("voicevox_core"));

        unsafe {
            let lib = &Library::new(cdylib_path)?;
            let symbols = Symbols::new(lib)?;

            match self {
                Self::CompatibleEngine => compatible_engine::exec(symbols)?,
                Self::VoicevoxGetVersion => voicevox_get_version::exec(symbols)?,
            }
        }
        Ok(())
    }

    fn assert_output(self, output: Output) -> AssertResult {
        use operations::*;

        match self {
            Self::CompatibleEngine => compatible_engine::assert_output(output),
            Self::VoicevoxGetVersion => voicevox_get_version::assert_output(output),
        }
    }
}

impl From<Test> for Trial {
    fn from(test: Test) -> Self {
        Trial::test(<&str>::from(test).to_snake_case(), move || {
            let current_exe = process_path::get_executable_path()
                .ok_or("could not get the path of this process")?;

            let output = assert_cmd::Command::new(current_exe)
                .args(["--exec-voicevox-c-api-e2e-test", test.into()])
                .env(
                    "VV_MODELS_ROOT_DIR",
                    Path::new(env!("CARGO_WORKSPACE_DIR")).join("model"),
                )
                .output()?;

            test.assert_output(output)?;
            Ok(())
        })
        .with_ignored_flag(true)
    }
}

struct Symbols<'lib> {
    voicevox_get_version: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    initialize: Symbol<'lib, unsafe extern "C" fn(bool, c_int, bool) -> bool>,
    load_model: Symbol<'lib, unsafe extern "C" fn(i64) -> bool>,
    is_model_loaded: Symbol<'lib, unsafe extern "C" fn(i64) -> bool>,
    finalize: Symbol<'lib, unsafe extern "C" fn()>,
    metas: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    supported_devices: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    yukarin_s_forward:
        Symbol<'lib, unsafe extern "C" fn(i64, *mut i64, *mut i64, *mut f32) -> bool>,
}

impl<'lib> Symbols<'lib> {
    unsafe fn new(lib: &'lib Library) -> Result<Self, libloading::Error> {
        macro_rules! new(($($name:ident),* $(,)?) => {
            Self {
                $(
                    $name: lib.get(stringify!($name).as_ref())?,
                )*
            }
        });

        Ok(new!(
            voicevox_get_version,
            initialize,
            load_model,
            is_model_loaded,
            finalize,
            metas,
            supported_devices,
            yukarin_s_forward,
        ))
    }
}
