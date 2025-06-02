//! `voicevox_user_dict_delete`を二度呼ぶとクラッシュすることを確認する。

use std::sync::LazyLock;

use assert_cmd::assert::AssertResult;
use indexmap::IndexSet;
use libloading::Library;
use serde::{Deserialize, Serialize};
use test_util::c_api::CApi;

use crate::{
    assert_cdylib::{self, Utf8Output, case},
    snapshots,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "double_delete_user_dict")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        // SAFETY: The safety contract must be upheld by the caller.
        let lib = unsafe { CApi::from_library(lib) }?;

        // SAFETY: `voicevox_user_dict_new` and `voicevox_user_dict_delete` have no safety
        // requirements.
        let dict = unsafe { lib.voicevox_user_dict_new() };
        unsafe { lib.voicevox_user_dict_delete(dict) };
        unsafe { lib.voicevox_user_dict_delete(dict) };
        unreachable!();
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        let mut assert = output.assert().try_failure()?.try_stdout("")?;
        for s in &SNAPSHOTS.stderr_matches_all {
            let p = predicates::str::is_match(s).unwrap_or_else(|e| panic!("{e}"));
            assert = assert.try_stderr(p)?;
        }
        Ok(assert)
    }
}

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(double_delete_user_dict);

#[derive(Deserialize)]
struct Snapshots {
    stderr_matches_all: IndexSet<String>,
}
