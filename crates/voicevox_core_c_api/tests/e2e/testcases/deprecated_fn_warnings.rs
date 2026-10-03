use std::{ptr, str, sync::LazyLock};

use assert_cmd::assert::AssertResult;
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

#[typetag::serde(name = "deprecated_fn_warnings")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        // SAFETY: The safety contract must be upheld by the caller.
        let lib = unsafe { CApi::from_library(lib) }?;

        // SAFETY: They have no safety requirements.
        unsafe { lib.voicevox_json_free(ptr::null_mut()) };
        unsafe { lib.voicevox_json_free(ptr::null_mut()) };
        unsafe { lib.voicevox_wav_free(ptr::null_mut()) };
        unsafe { lib.voicevox_wav_free(ptr::null_mut()) };
        Ok(())
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        output
            .mask_timestamps()
            .mask_unix_onnxruntime_filename()
            .mask_windows_video_cards()
            .assert()
            .try_success()?
            .try_stdout("")?
            .try_stderr(&*SNAPSHOTS.stderr)
    }
}

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(deprecated_fn_warnings);

#[derive(Deserialize)]
struct Snapshots {
    stderr: String,
}
