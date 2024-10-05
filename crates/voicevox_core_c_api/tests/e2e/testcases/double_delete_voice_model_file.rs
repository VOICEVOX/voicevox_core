//! `voicevox_voice_model_file_close`を二度呼ぶとクラッシュすることを確認する。

use std::{mem::MaybeUninit, sync::LazyLock};

use assert_cmd::assert::AssertResult;
use indexmap::IndexSet;
use libloading::Library;
use serde::{Deserialize, Serialize};
use test_util::c_api::{self, CApi, VoicevoxResultCode};

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "double_delete_voice_model_file")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        let lib = CApi::from_library(lib)?;

        let model = {
            let mut model = MaybeUninit::uninit();
            assert_ok(lib.voicevox_voice_model_file_open(
                c_api::SAMPLE_VOICE_MODEL_FILE_PATH.as_ptr(),
                model.as_mut_ptr(),
            ));
            model.assume_init()
        };

        lib.voicevox_voice_model_file_close(model);
        lib.voicevox_voice_model_file_close(model);
        unreachable!();

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK, result_code);
        }
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        let mut assert = output.assert().try_failure()?.try_stdout("")?;
        for s in &SNAPSHOTS.stderr_contains_all {
            assert = assert.try_stderr(predicates::str::contains(s))?;
        }
        Ok(assert)
    }
}

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(double_delete_voice_model_file);

#[derive(Deserialize)]
struct Snapshots {
    stderr_contains_all: IndexSet<String>,
}
