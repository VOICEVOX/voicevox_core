use std::{ffi::c_char, path::Path};

use assert_cmd::assert::{Assert, AssertResult};
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
        cmd!(
            env!("CARGO"),
            "build",
            "-p",
            env!("CARGO_PKG_NAME"),
            "--lib",
        )
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
    Metas,
    VoicevoxGetVersion,
}

impl Test {
    fn exec(self) -> anyhow::Result<()> {
        use operations::*;

        let cdylib_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("target")
            .join("debug")
            .join(libloading::library_filename("voicevox_core"));

        unsafe {
            let lib = &Library::new(cdylib_path)?;
            let symbols = Symbols::new(lib)?;

            match self {
                Self::VoicevoxGetVersion => voicevox_get_version::exec(symbols)?,
                Self::Metas => metas::exec(symbols)?,
            }
        }
        Ok(())
    }

    fn assert_output(self, assert: Assert) -> AssertResult {
        use operations::*;

        match self {
            Self::VoicevoxGetVersion => voicevox_get_version::assert_output(assert),
            Self::Metas => metas::assert_output(assert),
        }
    }
}

impl From<Test> for Trial {
    fn from(test: Test) -> Self {
        Trial::test(<&str>::from(test).to_snake_case(), move || {
            let current_exe = process_path::get_executable_path()
                .ok_or("could not get the path of this process")?;

            let assert = assert_cmd::Command::new(current_exe)
                .args(["--exec-voicevox-c-api-e2e-test", test.into()])
                .env(
                    "VV_MODELS_ROOT_DIR",
                    Path::new(env!("CARGO_WORKSPACE_DIR")).join("model"),
                )
                .assert();

            test.assert_output(assert)?;
            Ok(())
        })
        .with_ignored_flag(true)
    }
}

struct Symbols<'lib> {
    metas: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    voicevox_get_version: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
}

impl<'lib> Symbols<'lib> {
    unsafe fn new(lib: &'lib Library) -> Result<Self, libloading::Error> {
        let metas = lib.get(b"metas")?;
        let voicevox_get_version = lib.get(b"voicevox_get_version")?;

        Ok(Self {
            metas,
            voicevox_get_version,
        })
    }
}
