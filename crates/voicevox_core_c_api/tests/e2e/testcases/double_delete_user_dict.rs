//! `voicevox_user_dict_delete`を二度呼ぶとクラッシュすることを確認する。

use std::sync::LazyLock;

use assert_cmd::assert::AssertResult;
use indexmap::IndexSet;
use libloading::Library;
use serde::{Deserialize, Serialize};
use test_util::c_api::CApi;

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "double_delete_user_dict")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        let lib = CApi::from_library(lib)?;

        let dict = lib.voicevox_user_dict_new();
        lib.voicevox_user_dict_delete(dict);
        lib.voicevox_user_dict_delete(dict);
        unreachable!();
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        let mut assert = output.assert().try_failure()?.try_stdout("")?;
        for s in &SNAPSHOTS.stderr_contains_all {
            assert = assert.try_stderr(predicates::str::contains(s))?;
        }
        Ok(assert)
    }
}

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(double_delete_user_dict);

#[derive(Deserialize)]
struct Snapshots {
    stderr_contains_all: IndexSet<String>,
}
