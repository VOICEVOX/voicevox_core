use std::{
    path::{Path, PathBuf},
    process::{ExitStatus, Output},
};

use anyhow::{Context as _, ensure};
use assert_cmd::assert::{Assert, AssertResult, OutputAssertExt as _};
use clap::Parser as _;
use duct::cmd;
use easy_ext::ext;
use itertools::Itertools as _;
use libloading::Library;
use libtest_mimic::{Failed, Trial};

// assert_cmdのようにDLLをテストする。
// ただしstdout/stderrをキャプチャするため、DLLの実行自体は別プロセスで行う。
// テスト情報である`TestCase`をJSONにして本バイナリ自身を再帰的に呼ぶことで、プロセス分離を実現している。

/// `TestCase`の具体値をグローバルに登録する。
///
/// 式はconstでなくてもよい。
macro_rules! case {
    ($testcase:expr $(,)?) => {
        ::inventory::submit!(crate::assert_cdylib::TestCaseSubmission(|| Box::new(
            $testcase,
        )));
    };
}
pub(crate) use case;

/// 全てのテストを実行する。
pub(crate) fn exec<C: TestContext>() -> anyhow::Result<()> {
    if let Ok(AlternativeArguments {
        exec_c_api_e2e_test,
    }) = clap::Parser::try_parse()
    {
        let exec_c_api_e2e_test = serde_json::from_str::<Box<dyn TestCase>>(&exec_c_api_e2e_test)?;

        return unsafe {
            let lib = Library::new(C::cdylib_path())?;
            exec_c_api_e2e_test.exec(lib)
        };
    }

    let args = &libtest_mimic::Arguments::parse();

    // テスト対象が無いときに`cargo build`をスキップしたいが、判定部分がプライベート。
    // そのためスキップするのはCLIオプションに`--ignored`か`--include-ignored`が無いときのみ
    if args.ignored || args.include_ignored {
        cmd!(
            env!("CARGO"),
            "build",
            "--release",
            "--lib",
            "--features",
            &format!(",{}", C::FEATURES.iter().format(",")),
        )
        .run()?;

        ensure!(
            C::cdylib_path().exists(),
            "{} should exist",
            C::cdylib_path().display(),
        );
    }

    let tests = inventory::iter()
        .map(|TestCaseSubmission(testcase)| C::build_test(testcase()))
        .collect::<Result<_, _>>()?;

    libtest_mimic::run(args, tests).exit();

    #[derive(clap::Parser)]
    struct AlternativeArguments {
        #[arg(long, required(true))]
        exec_c_api_e2e_test: String,
    }

    #[ext]
    impl<C: TestContext> C {
        fn cdylib_path() -> PathBuf {
            Path::new(Self::TARGET_DIR)
                .join("release")
                .join(libloading::library_filename(Self::CDYLIB_NAME))
        }

        fn build_test(testcase: Box<dyn TestCase>) -> anyhow::Result<Trial> {
            let json = serde_json::to_string(&testcase)?;
            let current_exe = process_path::get_executable_path()
                .with_context(|| "could not get the path of this process")?;

            Ok(Trial::test(json.to_string(), move || {
                let output = assert_cmd::Command::new(current_exe)
                    .args(["--exec-c-api-e2e-test", &json])
                    .envs(Self::RUNTIME_ENVS.iter().copied())
                    .output()?
                    .try_into()?;

                testcase.assert_output(output)?;
                Ok(())
            })
            .with_ignored_flag(true))
        }
    }
}

pub(crate) trait TestContext {
    const FEATURES: &'static [&'static str];
    const TARGET_DIR: &'static str;
    const CDYLIB_NAME: &'static str;
    const RUNTIME_ENVS: &'static [(&'static str, &'static str)];
}

/// 個別のテストケースのインターフェイス。
#[typetag::serde(tag = "name")]
pub(crate) trait TestCase: Send {
    /// cdylibに対して操作を行う。
    ///
    /// `exec`は独立したプロセスで実行されるため、stdout/stderrへの出力をしたりグローバルな状態に
    /// 依存してもよい。
    ///
    /// # Safety
    ///
    /// `lib`は[`test_util::c_api::CApi`]として正しい動的ライブラリでなければならない。
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()>;

    /// 別プロセスで実行された`exec`の結果をチェックする。
    #[expect(clippy::result_large_err, reason = "多分assert_cmdの責務")]
    fn assert_output(&self, output: Utf8Output) -> AssertResult;
}

pub(crate) struct TestCaseSubmission(pub(crate) fn() -> Box<dyn TestCase>);

// これに登録された構造体が実行される。
inventory::collect!(TestCaseSubmission);

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
