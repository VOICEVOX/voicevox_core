// エンジンを起動してyukarin_s・yukarin_sa・decodeの推論を行う

use std::ffi::CStr;

use assert_cmd::assert::AssertResult;
use libloading::Library;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use voicevox_core::SupportedDevices;

use test_util::{c_api::CApi, EXAMPLE_DATA};

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    float_assert, snapshots,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "compatible_engine")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        let lib = CApi::from_library(lib)?;

        let metas_json = {
            let metas_json = lib.metas();
            let metas_json = CStr::from_ptr(metas_json).to_str()?;
            serde_json::to_string_pretty(&metas_json.parse::<serde_json::Value>()?).unwrap()
        };

        {
            let supported_devices = lib.supported_devices();
            serde_json::from_str::<SupportedDevices>(CStr::from_ptr(supported_devices).to_str()?)?;
        }

        assert!(lib.initialize(false, 0, false));

        assert!(!lib.is_model_loaded(EXAMPLE_DATA.speaker_id));
        assert!(lib.load_model(EXAMPLE_DATA.speaker_id));
        assert!(lib.is_model_loaded(EXAMPLE_DATA.speaker_id));

        // テスト用テキストは"t e s u t o"
        let phoneme_length = {
            let mut phoneme_length = [0.; 8];
            assert!(lib.yukarin_s_forward(
                EXAMPLE_DATA.duration.length,
                EXAMPLE_DATA.duration.phoneme_vector.as_ptr() as *mut i64,
                &mut { EXAMPLE_DATA.speaker_id } as *mut i64,
                phoneme_length.as_mut_ptr(),
            ));
            phoneme_length
        };

        let intonation_list = {
            let mut intonation_list = [0.; 5];
            assert!(lib.yukarin_sa_forward(
                EXAMPLE_DATA.intonation.length,
                EXAMPLE_DATA.intonation.vowel_phoneme_vector.as_ptr() as *mut i64,
                EXAMPLE_DATA.intonation.consonant_phoneme_vector.as_ptr() as *mut i64,
                EXAMPLE_DATA.intonation.start_accent_vector.as_ptr() as *mut i64,
                EXAMPLE_DATA.intonation.end_accent_vector.as_ptr() as *mut i64,
                EXAMPLE_DATA.intonation.start_accent_phrase_vector.as_ptr() as *mut i64,
                EXAMPLE_DATA.intonation.end_accent_phrase_vector.as_ptr() as *mut i64,
                &mut { EXAMPLE_DATA.speaker_id } as *mut i64,
                intonation_list.as_mut_ptr(),
            ));
            intonation_list
        };

        let wave = {
            let mut wave = vec![0.; 256 * EXAMPLE_DATA.decode.f0_length as usize];
            assert!(lib.decode_forward(
                EXAMPLE_DATA.decode.f0_length,
                EXAMPLE_DATA.decode.phoneme_size,
                EXAMPLE_DATA.decode.f0_vector.as_ptr() as *mut f32,
                EXAMPLE_DATA.decode.phoneme_vector.as_ptr() as *mut f32,
                &mut { EXAMPLE_DATA.speaker_id } as *mut i64,
                wave.as_mut_ptr(),
            ));
            wave
        };

        std::assert_eq!(SNAPSHOTS.metas, metas_json);

        float_assert::close_l1(&phoneme_length, &EXAMPLE_DATA.duration.result, 0.01);
        float_assert::close_l1(&intonation_list, &EXAMPLE_DATA.intonation.result, 0.01);

        assert!(wave.iter().copied().all(f32::is_normal));

        lib.finalize();
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

static SNAPSHOTS: Lazy<Snapshots> = snapshots::section!(compatible_engine);

#[derive(Deserialize)]
struct Snapshots {
    metas: String,
    #[serde(deserialize_with = "snapshots::deserialize_platform_specific_snapshot")]
    stderr: String,
}
