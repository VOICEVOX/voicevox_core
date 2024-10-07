//! `voicevox_open_jtalk_rc_delete`を二度呼ぶとクラッシュすることを確認する。

use std::{ffi::CString, mem::MaybeUninit, sync::LazyLock};

use assert_cmd::assert::AssertResult;
use indexmap::IndexSet;
use libloading::Library;
use serde::{Deserialize, Serialize};
use test_util::{
    c_api::{self, CApi, VoicevoxResultCode},
    OPEN_JTALK_DIC_DIR,
};

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "double_delete_openjtalk")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        let lib = CApi::from_library(lib)?;

        let openjtalk = {
            let mut openjtalk = MaybeUninit::uninit();
            let open_jtalk_dic_dir = CString::new(OPEN_JTALK_DIC_DIR).unwrap();
            assert_ok(
                lib.voicevox_open_jtalk_rc_new(open_jtalk_dic_dir.as_ptr(), openjtalk.as_mut_ptr()),
            );
            openjtalk.assume_init()
        };

        lib.voicevox_open_jtalk_rc_delete(openjtalk);
        lib.voicevox_open_jtalk_rc_delete(openjtalk);
        unreachable!();

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK, result_code);
        }
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

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(double_delete_openjtalk);

#[derive(Deserialize)]
struct Snapshots {
    stderr_matches_all: IndexSet<String>,
}
