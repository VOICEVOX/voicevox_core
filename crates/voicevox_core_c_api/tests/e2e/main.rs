use std::{
    ffi::{c_char, c_int},
    path::Path,
    process::{ExitStatus, Output},
};

use assert_cmd::assert::{Assert, AssertResult, OutputAssertExt as _};
use clap::{Parser as _, ValueEnum};
use duct::cmd;
use easy_ext::ext;
use heck::ToSnakeCase as _;
use libloading::{Library, Symbol};
use libtest_mimic::{Failed, Trial};
use once_cell::sync::Lazy;
use regex::{Regex, Replacer};
use strum::IntoStaticStr;

mod operations;

fn main() -> anyhow::Result<()> {
    if let Ok(ExecCApiE2eTest {
        exec_c_api_e2e_test,
    }) = ExecCApiE2eTest::try_parse()
    {
        return exec_c_api_e2e_test.exec();
    }

    let args = &libtest_mimic::Arguments::parse();

    // テスト対象が無いときに`cargo build`をスキップしたいが、判定部分がプライベート。
    // そのためスキップするのはCLIオプションに`--ignored`か`--include-ignored`が無いときのみ
    if args.ignored || args.include_ignored {
        cmd!(env!("CARGO"), "build", "--lib")
            // 他の単体テストが動いているときにonnxruntime-sysの初回ビルドを行うと、Windows環境だと
            // `$ORT_OUT_DIR`のハックが問題を起こす。そのためこのハック自体を無効化する
            //
            // featuresの差分を出さないように`cargo build`することができればonnxruntime-sysの
            // ビルド自体がされないのだが、このバイナリから`cargo build`の状況を知るのは無理に近い
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
struct ExecCApiE2eTest {
    #[arg(long, required(true))]
    exec_c_api_e2e_test: Test,
}

#[derive(Clone, Copy, ValueEnum, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
enum Test {
    CompatibleEngine,
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
            }
        }
        Ok(())
    }

    fn assert_output(self, output: Utf8Output) -> AssertResult {
        use operations::*;

        match self {
            Self::CompatibleEngine => compatible_engine::assert_output(output),
        }
    }
}

impl From<Test> for Trial {
    fn from(test: Test) -> Self {
        Trial::test(<&str>::from(test).to_snake_case(), move || {
            let current_exe = process_path::get_executable_path()
                .ok_or("could not get the path of this process")?;

            let output = assert_cmd::Command::new(current_exe)
                .args(["--exec-c-api-e2e-test", test.into()])
                .env(
                    "VV_MODELS_ROOT_DIR",
                    Path::new("..").join("..").join("model"),
                )
                .output()?
                .try_into()?;

            test.assert_output(output)?;
            Ok(())
        })
        .with_ignored_flag(true)
    }
}

struct Utf8Output {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

impl Utf8Output {
    fn assert(self) -> Assert {
        Output::from(self).assert()
    }
}

const _: () = {
    macro_rules! static_regex {
        ($regex:expr $(,)?) => {{
            static REGEX: Lazy<Regex> = Lazy::new(|| $regex.parse().unwrap());
            &REGEX
        }};
    }

    impl Utf8Output {
        fn mask_timestamps(self) -> Self {
            self.mask_stderr(
                static_regex!(
                    "(?m)^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}.[0-9]{6}Z",
                ),
                "{timestamp}",
            )
        }

        fn mask_windows_video_cards(self) -> Self {
            self.mask_stderr(
                static_regex!(
                    r#"(?m)^\{timestamp\}  INFO voicevox_core::publish: 検出されたGPU \(DirectMLには1番目のGPUが使われます\):(\n\{timestamp\}  INFO voicevox_core::publish:   - "[^"]+" \([a-zA-Z0-9 ]+\))+"#,
                ),
                "{windows-video-cards}",
            )
        }
    }

    #[ext]
    impl Utf8Output {
        fn mask_stderr(self, regex: &Regex, rep: impl Replacer) -> Self {
            let stderr = regex.replace_all(&self.stderr, rep).into_owned();
            Self { stderr, ..self }
        }
    }
};

impl TryFrom<Output> for Utf8Output {
    type Error = Failed;

    fn try_from(
        Output {
            status,
            stdout,
            stderr,
        }: Output,
    ) -> Result<Self, Self::Error> {
        let stdout = String::from_utf8(stdout)?;
        let stderr = String::from_utf8(stderr)?;
        Ok(Self {
            status,
            stdout,
            stderr,
        })
    }
}

impl From<Utf8Output> for Output {
    fn from(
        Utf8Output {
            status,
            stdout,
            stderr,
        }: Utf8Output,
    ) -> Self {
        Self {
            status,
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
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
