use std::{ffi::CStr, mem::MaybeUninit};

use assert_cmd::assert::AssertResult;
use indoc::indoc;
use libloading::Library;
use serde::{Deserialize, Serialize};
use test_util::c_api::{self, CApi, VoicevoxResultCode};

use crate::assert_cdylib::{self, Utf8Output, case};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "audio_query_frame_length")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        // SAFETY: The safety contract must be upheld by the caller.
        let lib = unsafe { CApi::from_library(lib) }?;

        static QUERY: &CStr = indoc! {cr#"
            {
                "accent_phrases": [
                    {
                        "moras": [
                            {
                                "text": "ア",
                                "vowel": "a",
                                "vowel_length": 4.4,
                                "pitch": 0.0
                            }
                        ],
                        "accent": 1
                    }
                ],
                "speedScale": 1.2,
                "pitchScale": 0.0,
                "intonationScale": 1.0,
                "volumeScale": 1.0,
                "prePhonemeLength": 3.3,
                "postPhonemeLength": 5.5,
                "outputSamplingRate": 24000,
                "outputStereo": false
            }
        "#};

        let frame_length = {
            let mut frame_length = MaybeUninit::uninit();
            unsafe {
                // SAFETY: The safety contract must be upheld by the caller.
                assert_ok(lib.voicevox_audio_query_frame_length(
                    QUERY.as_ptr(),
                    lib.voicevox_make_default_audio_query_frame_length_options(),
                    frame_length.as_mut_ptr(),
                ));
            };
            // SAFETY: `voicevox_audio_query_frame_length` initializes `frame_length` if succeeded.
            unsafe { frame_length.assume_init() }
        };

        let to_frame_length =
            |secs: f32| ((secs * 93.75).round_ties_even() / 1.2).round_ties_even() as usize;
        std::assert_eq!(
            to_frame_length(3.3) + to_frame_length(4.4) + to_frame_length(5.5),
            frame_length,
        );

        return Ok(());

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK, result_code);
        }
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        output
            .mask_timestamps()
            .mask_unix_onnxruntime_filename()
            .mask_windows_video_cards()
            .assert()
            .try_success()?
            .try_stdout("")?
            .try_stderr("")
    }
}
