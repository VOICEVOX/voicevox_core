use std::{collections::HashMap, ffi::CStr, mem::MaybeUninit, str, sync::LazyLock};

use assert_cmd::assert::AssertResult;
use libloading::Library;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use test_util::c_api::{self, CApi, VoicevoxResultCode};
use voicevox_core::SupportedDevices;

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "global_info")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        let lib = CApi::from_library(lib)?;

        std::assert_eq!(
            env!("CARGO_PKG_VERSION"),
            CStr::from_ptr(lib.voicevox_get_version()).to_str()?,
        );

        let onnxruntime = {
            let mut onnxruntime = MaybeUninit::uninit();
            assert_ok(lib.voicevox_onnxruntime_load_once(
                lib.voicevox_make_default_load_onnxruntime_options(),
                onnxruntime.as_mut_ptr(),
            ));
            onnxruntime.assume_init()
        };

        {
            let mut supported_devices = MaybeUninit::uninit();
            assert_ok(lib.voicevox_onnxruntime_create_supported_devices_json(
                onnxruntime,
                supported_devices.as_mut_ptr(),
            ));
            let supported_devices = supported_devices.assume_init();
            serde_json::from_str::<SupportedDevices>(CStr::from_ptr(supported_devices).to_str()?)?;
            lib.voicevox_json_free(supported_devices);
        }

        for result_code in [
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_GPU_SUPPORT_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_STYLE_NOT_FOUND_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_MODEL_NOT_FOUND_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_ML_INFERENCE_ERROR,
            c_api::VoicevoxResultCode_VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR,
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
                str::from_utf8(
                    CStr::from_ptr(lib.voicevox_error_result_to_message(result_code)).to_bytes()
                )?,
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
            .mask_onnxruntime_version()
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
