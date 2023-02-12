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
        yukarin_sa_forward,
        decode_forward,
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

    // "テスト"

    let mut phoneme_length = [0.; 8];
    assert!(yukarin_s_forward(
        8,
        [0, 37, 14, 35, 6, 37, 30, 0].as_mut_ptr(),
        &mut { SPEAKER_ID } as *mut i64,
        phoneme_length.as_mut_ptr(),
    ));
    std::assert_eq!(
        SNAPSHOTS.compatible_engine.yukarin_s_forward,
        Sha256Sum::le_bytes(&phoneme_length),
    );

    let mut intonation_list = [0.; 5];
    assert!(yukarin_sa_forward(
        5,
        [0, 14, 6, 30, 0].as_mut_ptr(),
        [-1, 37, 35, 37, -1].as_mut_ptr(),
        [0, 1, 0, 0, 0].as_mut_ptr(),
        [0, 1, 0, 0, 0].as_mut_ptr(),
        [0, 1, 0, 0, 0].as_mut_ptr(),
        [0, 0, 0, 1, 0].as_mut_ptr(),
        &mut { SPEAKER_ID } as *mut i64,
        intonation_list.as_mut_ptr(),
    ));
    std::assert_eq!(
        SNAPSHOTS.compatible_engine.yukarin_sa_forward,
        Sha256Sum::le_bytes(&intonation_list),
    );

    let mut wave = [0.; 256 * F0_LENGTH];
    assert!(decode_forward(
        F0_LENGTH as _,
        PHONEME_SIZE as _,
        {
            let mut f0 = [0.; 69];
            f0[9..24].fill(5.905218);
            f0[37..60].fill(5.565851);
            f0
        }
        .as_mut_ptr(),
        {
            let mut phoneme = [0.; PHONEME_SIZE * F0_LENGTH];
            let mut set_one = |index, range| {
                for i in range {
                    phoneme[i * PHONEME_SIZE + index] = 1.;
                }
            };
            set_one(0, 0..9);
            set_one(37, 9..13);
            set_one(14, 13..24);
            set_one(35, 24..30);
            set_one(6, 30..37);
            set_one(37, 37..45);
            set_one(30, 45..60);
            set_one(0, 60..69);
            phoneme
        }
        .as_mut_ptr(),
        &mut { SPEAKER_ID } as *mut i64,
        wave.as_mut_ptr(),
    ));
    std::assert_eq!(
        SNAPSHOTS.compatible_engine.decode_forward,
        Sha256Sum::le_bytes(&wave),
    );

    finalize();
    return Ok(());

    const SPEAKER_ID: i64 = 0;
    const F0_LENGTH: usize = 69;
    const PHONEME_SIZE: usize = 45;
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
    yukarin_sa_forward: Sha256Sum,
    decode_forward: Sha256Sum,
    #[serde(deserialize_with = "super::deserialize_platform_specific_snapshot")]
    stderr: String,
}
