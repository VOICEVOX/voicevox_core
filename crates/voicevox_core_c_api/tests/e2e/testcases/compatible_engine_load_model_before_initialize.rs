// initialize前にモデルを読み込むとエラーになるテスト

use std::ffi::CStr;

use assert_cmd::assert::AssertResult;
use libloading::Library;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use test_util::c_api::CApi;

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "compatible_engine_load_model_before_initialize")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        let lib = CApi::from_library(lib)?;

        assert!(!lib.load_model(0));
        let last_error_message = lib.last_error_message();
        let last_error_message = CStr::from_ptr(last_error_message).to_str()?;

        std::assert_eq!(SNAPSHOTS.last_error_message, last_error_message);
        Ok(())
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        output
            .mask_timestamps()
            .mask_onnxruntime_version()
            .mask_windows_video_cards()
            .assert()
            .try_success()?
            .try_stdout("")?
            .try_stderr(&*SNAPSHOTS.stderr)
    }
}

static SNAPSHOTS: Lazy<Snapshots> =
    snapshots::section!(compatible_engine_load_model_before_initialize);

#[derive(Deserialize)]
struct Snapshots {
    last_error_message: String,
    stderr: String,
}
