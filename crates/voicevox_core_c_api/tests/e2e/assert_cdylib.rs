use std::{
    path::{Path, PathBuf},
    process::{ExitStatus, Output},
};

use anyhow::{anyhow, ensure};
use assert_cmd::assert::{Assert, AssertResult, OutputAssertExt as _};
use clap::Parser as _;
use duct::cmd;
use easy_ext::ext;
use libloading::Library;
use libtest_mimic::{Failed, Trial};
use serde::{de::DeserializeOwned, Serialize};

// assert_cmdのようにDLLをテストする。
// ただしstdout/stderrをキャプチャするため、DLLの実行自体は別プロセスで行う。
// テスト情報である`TestCase`をJSONにして本バイナリ自身を再帰的に呼ぶことで、プロセス分離を実現している。

pub(crate) fn exec<T: TestSuite>() -> anyhow::Result<()> {
    if let Ok(AlternativeArguments {
        exec_c_api_e2e_test,
    }) = clap::Parser::try_parse()
    {
        let exec_c_api_e2e_test = serde_json::from_str(&exec_c_api_e2e_test)?;

        return unsafe {
            let lib = &Library::new(T::cdylib_path())?;
            T::exec(exec_c_api_e2e_test, lib)
        };
    }

    let args = &libtest_mimic::Arguments::parse();

    // テスト対象が無いときに`cargo build`をスキップしたいが、判定部分がプライベート。
    // そのためスキップするのはCLIオプションに`--ignored`か`--include-ignored`が無いときのみ
    if args.ignored || args.include_ignored {
        let mut cmd = cmd!(env!("CARGO"), "build", "--lib");
        for (k, v) in T::build_envs() {
            cmd = cmd.env(k, v);
        }
        cmd.run()?;

        ensure!(
            T::cdylib_path().exists(),
            "{} should exist",
            T::cdylib_path().display(),
        );
    }

    let tests = T::testcases()
        .into_iter()
        .map(T::build_test)
        .collect::<Result<_, _>>()?;

    libtest_mimic::run(args, tests).exit();

    #[derive(clap::Parser)]
    struct AlternativeArguments {
        #[arg(long, required(true))]
        exec_c_api_e2e_test: String,
    }

    #[ext]
    impl<T: TestSuite> T {
        fn cdylib_path() -> PathBuf {
            Path::new(Self::TARGET_DIR)
                .join("debug")
                .join(libloading::library_filename(Self::CDYLIB_NAME))
        }

        fn build_test(testcase: Self::TestCase) -> anyhow::Result<Trial> {
            let json = serde_json::to_string(&testcase)?;
            let current_exe = process_path::get_executable_path()
                .ok_or_else(|| anyhow!("could not get the path of this process"))?;

            Ok(Trial::test(json.to_string(), move || {
                let output = assert_cmd::Command::new(current_exe)
                    .args(["--exec-c-api-e2e-test", &json])
                    .envs(Self::runtime_envs().iter().copied())
                    .output()?
                    .try_into()?;

                Self::assert_output(testcase, output)?;
                Ok(())
            })
            .with_ignored_flag(true))
        }
    }
}

pub(crate) trait TestSuite {
    type TestCase: Serialize + DeserializeOwned + Send + 'static;

    const TARGET_DIR: &'static str;
    const CDYLIB_NAME: &'static str;

    fn build_envs() -> &'static [(&'static str, &'static str)];
    fn runtime_envs() -> &'static [(&'static str, &'static str)];
    fn testcases() -> Vec<Self::TestCase>;
    unsafe fn exec(testcase: Self::TestCase, lib: &Library) -> anyhow::Result<()>;
    fn assert_output(testcase: Self::TestCase, output: Utf8Output) -> AssertResult;
}

pub(crate) struct Utf8Output {
    pub(crate) status: ExitStatus,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

impl Utf8Output {
    pub(crate) fn assert(self) -> Assert {
        Output::from(self).assert()
    }
}

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
