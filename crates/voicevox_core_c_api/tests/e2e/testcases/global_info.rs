use std::ffi::CStr;

use assert_cmd::assert::AssertResult;
use libloading::Library;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use voicevox_core::{result_code::VoicevoxResultCode, SupportedDevices};

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
    symbols::Symbols,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "global_info")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: &Library) -> anyhow::Result<()> {
        let Symbols {
            voicevox_version,
            voicevox_get_supported_devices_json,
            voicevox_error_result_to_message,
            ..
        } = Symbols::new(lib)?;

        std::assert_eq!(
            voicevox_core::version!(),
            CStr::from_ptr(**voicevox_version).to_str()?,
        );

        std::assert_eq!(
            SupportedDevices::get_supported_devices()?.to_json(),
            CStr::from_ptr(voicevox_get_supported_devices_json())
                .to_str()?
                .parse::<serde_json::Value>()?,
        );

        for result_code in VoicevoxResultCode::iter() {
            std::assert_eq!(
                voicevox_core::result_code::error_result_to_message(result_code).as_bytes(),
                CStr::from_ptr(voicevox_error_result_to_message(result_code)).to_bytes_with_nul(),
            );
        }
        Ok(())
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        output
            .mask_timestamps()
            .mask_windows_video_cards()
            .assert()
            .try_success()?
            .try_stdout("")?
            .try_stderr(&*SNAPSHOTS.stderr)
    }
}

static SNAPSHOTS: Lazy<Snapshots> = snapshots::section!(global_info);

#[derive(Deserialize)]
struct Snapshots {
    stderr: String,
}
