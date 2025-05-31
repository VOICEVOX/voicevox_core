use std::{ptr, str};

use assert_cmd::assert::AssertResult;
use libloading::Library;
use serde::{Deserialize, Serialize};
use test_util::c_api::CApi;

use crate::assert_cdylib::{self, Utf8Output, case};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "free_for_null")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        // SAFETY: The safety contract must be upheld by the caller.
        let lib = unsafe { CApi::from_library(lib) }?;

        // SAFETY: `voicevox_json_free`, `voicevox_wav_free`, `voicevox_open_jtalk_rc_delete`,
        // `voicevox_synthesizer_delete`, `voicevox_voice_model_file_delete`, and
        // `voicevox_user_dict_delete` have no safety requirements.
        unsafe { lib.voicevox_json_free(ptr::null_mut()) };
        unsafe { lib.voicevox_wav_free(ptr::null_mut()) };
        unsafe { lib.voicevox_open_jtalk_rc_delete(ptr::null_mut()) };
        unsafe { lib.voicevox_synthesizer_delete(ptr::null_mut()) };
        unsafe { lib.voicevox_voice_model_file_delete(ptr::null_mut()) };
        unsafe { lib.voicevox_user_dict_delete(ptr::null_mut()) };
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
            .try_stderr("")
    }
}
