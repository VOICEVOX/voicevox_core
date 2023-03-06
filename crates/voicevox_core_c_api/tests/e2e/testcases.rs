use std::ffi::CStr;

use assert_cmd::assert::AssertResult;
use libloading::Library;
use serde::{Deserialize, Serialize};

use crate::{
    assert_cdylib::{TestCase, Utf8Output},
    float_assert,
    snapshots::SNAPSHOTS,
    symbols::Symbols,
};

#[derive(Serialize, Deserialize)]
struct CompatibleEngine;

inventory::submit!(&CompatibleEngine as &dyn TestCase);

#[typetag::serde]
impl TestCase for CompatibleEngine {
    unsafe fn exec(&self, lib: &Library) -> anyhow::Result<()> {
        let Symbols {
            initialize,
            load_model,
            is_model_loaded,
            finalize,
            metas,
            supported_devices,
            yukarin_s_forward,
            yukarin_sa_forward,
            decode_forward,
        } = Symbols::new(lib)?;

        let metas_json = {
            let metas_json = metas();
            let metas_json = CStr::from_ptr(metas_json).to_str()?;
            metas_json.parse::<serde_json::Value>()?;
            metas_json
        };

        let supported_devices = {
            let supported_devices = supported_devices();
            CStr::from_ptr(supported_devices)
                .to_str()?
                .parse::<serde_json::Value>()?
        };

        assert!(initialize(false, 0, false));

        assert!(!is_model_loaded(SPEAKER_ID));
        assert!(load_model(SPEAKER_ID));
        assert!(is_model_loaded(SPEAKER_ID));

        // テスト用テキストは"t e s u t o"

        let phoneme_length = {
            let mut phoneme_length = [0.; 8];
            assert!(yukarin_s_forward(
                8,
                [0, 37, 14, 35, 6, 37, 30, 0].as_mut_ptr(),
                &mut { SPEAKER_ID } as *mut i64,
                phoneme_length.as_mut_ptr(),
            ));
            phoneme_length
        };

        let intonation_list = {
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
            intonation_list
        };

        let wave = {
            let mut wave = [0.; 256 * F0_LENGTH];
            assert!(decode_forward(
                F0_LENGTH as _,
                PHONEME_SIZE as _,
                {
                    let mut f0 = [0.; F0_LENGTH];
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
            wave
        };

        std::assert_eq!(include_str!("../../../../model/metas.json"), metas_json);

        std::assert_eq!(
            voicevox_core::SUPPORTED_DEVICES.to_json(),
            supported_devices,
        );

        float_assert::close_l1(
            &phoneme_length,
            &SNAPSHOTS.compatible_engine.yukarin_s_forward,
            0.01,
        );

        float_assert::close_l1(
            &intonation_list,
            &SNAPSHOTS.compatible_engine.yukarin_sa_forward,
            0.01,
        );

        assert!(wave.iter().copied().all(f32::is_normal));

        finalize();
        return Ok(());

        const SPEAKER_ID: i64 = 0;
        const F0_LENGTH: usize = 69;
        const PHONEME_SIZE: usize = 45;
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        output
            .mask_timestamps()
            .mask_windows_video_cards()
            .assert()
            .try_success()?
            .try_stdout("")?
            .try_stderr(&*SNAPSHOTS.compatible_engine.stderr)
    }
}
