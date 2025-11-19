use std::{collections::HashMap, env, ffi::CStr, mem::MaybeUninit, str, sync::LazyLock};

use assert_cmd::assert::AssertResult;
use const_format::concatcp;
use libloading::Library;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use test_util::c_api::{self, CApi, VoicevoxLoadOnnxruntimeOptions, VoicevoxResultCode};

use crate::{
    assert_cdylib::{self, Utf8Output, case},
    snapshots,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "global_info")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        // SAFETY: The safety contract must be upheld by the caller.
        let lib = unsafe { CApi::from_library(lib) }?;

        std::assert_eq!(
            env!("CARGO_PKG_VERSION"),
            // SAFETY: `voicevox_get_version` has no safety requirements, and returns a valid
            // string.
            unsafe { CStr::from_ptr(lib.voicevox_get_version()) }.to_str()?,
        );

        let onnxruntime = {
            let mut onnxruntime = MaybeUninit::uninit();
            assert_ok(unsafe {
                // SAFETY:
                // - A `CStr` is a valid string.
                // - `onnxruntime` is valid for writes.
                lib.voicevox_onnxruntime_load_once(
                    VoicevoxLoadOnnxruntimeOptions {
                        filename: CStr::from_bytes_with_nul(
                            concatcp!(
                                env::consts::DLL_PREFIX,
                                "onnxruntime",
                                env::consts::DLL_SUFFIX,
                                '\0'
                            )
                            .as_ref(),
                        )
                        .expect("this ends with nul")
                        .as_ptr(),
                    },
                    onnxruntime.as_mut_ptr(),
                )
            });
            // SAFETY: `voicevox_onnxruntime_load_once` initializes `onnxruntime` if succeeded.
            unsafe { onnxruntime.assume_init() }
        };

        {
            let supported_devices = {
                let mut supported_devices = MaybeUninit::uninit();
                assert_ok(unsafe {
                    // SAFETY:
                    // - `onnxruntime` is valid for reads.
                    // - `supported_devices` is valid for writes.
                    lib.voicevox_onnxruntime_create_supported_devices_json(
                        onnxruntime,
                        supported_devices.as_mut_ptr(),
                    )
                });
                // SAFETY: `voicevox_onnxruntime_create_supported_devices_json` initializes
                // `supported_devices` if succeeded.
                unsafe { supported_devices.assume_init() }
            };

            serde_json::from_str::<HashMap<String, bool>>(
                // SAFETY: `voicevox_onnxruntime_create_supported_devices_json` returns a valid
                // string if succeeded.
                unsafe { CStr::from_ptr(supported_devices) }.to_str()?,
            )?;

            // SAFETY: `supported_devices` is valid and is no longer used.
            unsafe { lib.voicevox_json_free(supported_devices) };
        }

        for result_code in [
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_GPU_SUPPORT_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_STYLE_NOT_FOUND_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_MODEL_NOT_FOUND_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_RUN_MODEL_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_ANALYZE_TEXT_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_PARSE_KANA_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_INVALID_ACCENT_PHRASE_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_OPEN_ZIP_FILE_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_READ_ZIP_ENTRY_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_MODEL_ALREADY_LOADED_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_STYLE_ALREADY_LOADED_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_INVALID_MODEL_DATA_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_LOAD_USER_DICT_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_SAVE_USER_DICT_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_USER_DICT_WORD_NOT_FOUND_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_USE_USER_DICT_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_INVALID_USER_DICT_WORD_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_INVALID_UUID_ERROR,
        ] {
            std::assert_eq!(
                SNAPSHOTS.result_messages[&result_code],
                // SAFETY: `voicevox_get_version` has safety requirement, and returns a valid
                // string.
                unsafe { CStr::from_ptr(lib.voicevox_error_result_to_message(result_code)) }
                    .to_str()?,
            );
        }
        return Ok(());

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK, result_code);
        }
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

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(global_info);

#[serde_as]
#[derive(Deserialize)]
struct Snapshots {
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    result_messages: HashMap<i32, String>,
    stderr: String,
}
