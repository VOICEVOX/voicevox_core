// エンジンを起動してyukarin_s・yukarin_sa・decodeの推論を行う

use std::ffi::CStr;

use assert_cmd::assert::AssertResult;
use libloading::Library;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use test_util::TestData;

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    float_assert, snapshots,
    symbols::Symbols,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "compatible_engine")]
impl assert_cdylib::TestCase for TestCase {
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
            ..
        } = Symbols::new(lib)?;

        let metas_json = {
            let metas_json = metas();
            let metas_json = CStr::from_ptr(metas_json).to_str()?;
            metas_json.parse::<serde_json::Value>()?;
            metas_json
        };

        let mut testdata = TestData::load();

        let supported_devices = {
            let supported_devices = supported_devices();
            CStr::from_ptr(supported_devices)
                .to_str()?
                .parse::<serde_json::Value>()?
        };

        assert!(initialize(false, 0, false));

        assert!(!is_model_loaded(testdata.speaker_id));
        assert!(load_model(testdata.speaker_id));
        assert!(is_model_loaded(testdata.speaker_id));

        // テスト用テキストは"t e s u t o"
        let phoneme_length = {
            let mut phoneme_length = [0.; 8];
            assert!(yukarin_s_forward(
                testdata.duration.length,
                testdata.duration.phoneme_vector.as_mut_ptr(),
                &mut { testdata.speaker_id } as *mut i64,
                phoneme_length.as_mut_ptr(),
            ));
            phoneme_length
        };

        let intonation_list = {
            let mut intonation_list = [0.; 5];
            assert!(yukarin_sa_forward(
                testdata.intonation.length,
                testdata.intonation.vowel_phoneme_vector.as_mut_ptr(),
                testdata.intonation.consonant_phoneme_vector.as_mut_ptr(),
                testdata.intonation.start_accent_vector.as_mut_ptr(),
                testdata.intonation.end_accent_vector.as_mut_ptr(),
                testdata.intonation.start_accent_phrase_vector.as_mut_ptr(),
                testdata.intonation.end_accent_phrase_vector.as_mut_ptr(),
                &mut { testdata.speaker_id } as *mut i64,
                intonation_list.as_mut_ptr(),
            ));
            intonation_list
        };

        let wave = {
            let mut wave = vec![0.; 256 * testdata.decode.f0_length as usize];
            assert!(decode_forward(
                testdata.decode.f0_length,
                testdata.decode.phoneme_size,
                testdata.decode.f0_vector.as_mut_ptr(),
                testdata.decode.phoneme_vector.as_mut_ptr(),
                &mut { testdata.speaker_id } as *mut i64,
                wave.as_mut_ptr(),
            ));
            wave
        };

        std::assert_eq!(include_str!("../../../../../model/metas.json"), metas_json);
        std::assert_eq!(
            voicevox_core::SUPPORTED_DEVICES.to_json(),
            supported_devices,
        );

        float_assert::close_l1(&phoneme_length, &testdata.duration.result, 0.01);
        float_assert::close_l1(&intonation_list, &testdata.intonation.result, 0.01);

        assert!(wave.iter().copied().all(f32::is_normal));

        finalize();
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

static SNAPSHOTS: Lazy<Snapshots> = snapshots::section!(compatible_engine);

#[derive(Deserialize)]
struct Snapshots {
    #[serde(deserialize_with = "snapshots::deserialize_platform_specific_snapshot")]
    stderr: String,
}
