// initialize前にモデルを読み込むとエラーになるテスト

use std::{ffi::CStr, sync::LazyLock};

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

#[typetag::serde(name = "compatible_engine_load_model_before_initialize")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        // SAFETY: The safety contract must be upheld by the caller.
        let lib = unsafe { CApi::from_library(lib) }?;

        // SAFETY: `load_model` has no safety requirements.
        assert!(unsafe { !lib.load_model(0) });

        // SAFETY: The string `last_error_message` remains valid until another error occurs.
        let last_error_message = unsafe { CStr::from_ptr(lib.last_error_message()) }.to_str()?;

        std::assert_eq!(SNAPSHOTS.last_error_message, last_error_message);
        Ok(())
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        output
            .mask_timestamps()
            .mask_onnxruntime_filename()
            .mask_windows_video_cards()
            .assert()
            .try_success()?
            .try_stdout("")?
            .try_stderr(&*SNAPSHOTS.stderr)
    }
}

static SNAPSHOTS: LazyLock<Snapshots> =
    snapshots::section!(compatible_engine_load_model_before_initialize);

#[derive(Deserialize)]
struct Snapshots {
    last_error_message: String,
    stderr: String,
}
