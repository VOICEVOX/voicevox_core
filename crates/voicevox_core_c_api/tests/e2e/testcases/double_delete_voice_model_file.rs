//! `voicevox_voice_model_file_delete`を二度呼ぶとクラッシュすることを確認する。

use std::{mem::MaybeUninit, sync::LazyLock};

use assert_cmd::assert::AssertResult;
use indexmap::IndexSet;
use libloading::Library;
use serde::{Deserialize, Serialize};
use test_util::c_api::{self, CApi, VoicevoxResultCode};

use crate::{
    assert_cdylib::{self, Utf8Output, case},
    snapshots,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "double_delete_voice_model_file")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        // SAFETY: The safety contract must be upheld by the caller.
        let lib = unsafe { CApi::from_library(lib) }?;

        let model = {
            let mut model = MaybeUninit::uninit();
            assert_ok(unsafe {
                // SAFETY:
                // - `SAMPLE_VOICE_MODEL_FILE_PATH` is a valid string.
                // - `model` is valid for writes.
                lib.voicevox_voice_model_file_open(
                    c_api::SAMPLE_VOICE_MODEL_FILE_PATH.as_ptr(),
                    model.as_mut_ptr(),
                )
            });
            // SAFETY: `voicevox_voice_model_file_open` initializes `model` if succeeded.
            unsafe { model.assume_init() }
        };

        // SAFETY: `voicevox_voice_model_file_delete` has no safety requirement.
        unsafe { lib.voicevox_voice_model_file_delete(model) };
        unsafe { lib.voicevox_voice_model_file_delete(model) };
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

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(double_delete_voice_model_file);

#[derive(Deserialize)]
struct Snapshots {
    stderr_matches_all: IndexSet<String>,
}
