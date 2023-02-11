use std::ffi::CStr;

use assert_cmd::assert::AssertResult;
use serde::Deserialize;

use crate::{Symbols, Utf8Output};

use super::{Sha256Sum, SNAPSHOTS};

pub(crate) unsafe fn exec(
    Symbols {
        initialize,
        load_model,
        is_model_loaded,
        finalize,
        metas,
        supported_devices,
        yukarin_s_forward,
    }: Symbols<'_>,
) -> anyhow::Result<()> {
    let metas_json = metas();
    let metas_json = CStr::from_ptr(metas_json).to_str()?;
    std::assert_eq!(include_str!("../../../../../model/metas.json"), metas_json);
    metas_json.parse::<serde_json::Value>()?;

    let supported_devices = supported_devices();
    let supported_devices = CStr::from_ptr(supported_devices)
        .to_str()?
        .parse::<serde_json::Value>()?;
    std::assert_eq!(
        voicevox_core::SUPPORTED_DEVICES.to_json(),
        supported_devices,
    );

    assert!(initialize(false, 0, false));

    assert!(!is_model_loaded(SPEAKER_ID));
    assert!(load_model(SPEAKER_ID));
    assert!(is_model_loaded(SPEAKER_ID));

    let mut phoneme_list = [0];
    let mut phoneme_length = [0.];
    assert!(yukarin_s_forward(
        phoneme_list.len() as _,
        phoneme_list.as_mut_ptr(),
        &mut { SPEAKER_ID } as *mut i64,
        phoneme_length.as_mut_ptr(),
    ));
    std::assert_eq!(
        SNAPSHOTS.compatible_engine.yukarin_s_forward,
        Sha256Sum::le_bytes(&phoneme_length),
    );

    finalize();
    return Ok(());

    const SPEAKER_ID: i64 = 0;
}

pub(crate) fn assert_output(output: Utf8Output) -> AssertResult {
    output
        .mask_timestamps()
        .mask_windows_video_cards()
        .assert()
        .try_success()?
        .try_stdout("")?
        .try_stderr(&*SNAPSHOTS.compatible_engine.stderr)
}

#[derive(Deserialize)]
pub(super) struct Snapshots {
    yukarin_s_forward: Sha256Sum,
    #[serde(deserialize_with = "super::deserialize_platform_specific_snapshot")]
    stderr: String,
}
