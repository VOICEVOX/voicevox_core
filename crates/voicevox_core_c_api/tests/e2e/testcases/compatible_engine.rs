// エンジンを起動してyukarin_s・yukarin_sa・decodeの推論を行う

use std::sync::LazyLock;
use std::{cmp::min, ffi::CStr};

use assert_cmd::assert::AssertResult;
use libloading::Library;
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

        // 中間生成物を経由した場合の生成音声
        let wave2 = {
            let length_with_margin =
                EXAMPLE_DATA.intermediate.f0_length + 2 * EXAMPLE_DATA.intermediate.margin_width;
            let mut audio_feature =
                vec![0.; (length_with_margin * EXAMPLE_DATA.intermediate.feature_dim) as usize];
            let mut wave = vec![0.; 256 * length_with_margin as usize];
            assert!(lib.generate_full_intermediate(
                EXAMPLE_DATA.intermediate.f0_length,
                EXAMPLE_DATA.intermediate.phoneme_size,
                EXAMPLE_DATA.intermediate.margin_width,
                EXAMPLE_DATA.intermediate.feature_dim,
                EXAMPLE_DATA.intermediate.f0_vector.as_ptr() as *mut f32,
                EXAMPLE_DATA.intermediate.phoneme_vector.as_ptr() as *mut f32,
                &mut { EXAMPLE_DATA.speaker_id } as *mut i64,
                audio_feature.as_mut_ptr(),
            ));
            assert!(lib.render_audio_segment(
                length_with_margin,
                EXAMPLE_DATA.intermediate.feature_dim,
                audio_feature.as_ptr() as *mut f32,
                &mut { EXAMPLE_DATA.speaker_id } as *mut i64,
                wave.as_mut_ptr(),
            ));
            wave[256 * EXAMPLE_DATA.intermediate.margin_width as usize
                ..wave.len() - 256 * EXAMPLE_DATA.intermediate.margin_width as usize]
                .to_vec()
        };

        // 中間生成物を経由し、さらにチャンクごとに変換した場合の生成音声
        let wave3 = {
            let length_with_margin =
                EXAMPLE_DATA.intermediate.f0_length + 2 * EXAMPLE_DATA.intermediate.margin_width;
            let mut audio_feature =
                vec![0.; (length_with_margin * EXAMPLE_DATA.intermediate.feature_dim) as usize];
            let mut wave = vec![0.; 256 * EXAMPLE_DATA.intermediate.f0_length as usize];
            assert!(lib.generate_full_intermediate(
                EXAMPLE_DATA.intermediate.f0_length,
                EXAMPLE_DATA.intermediate.phoneme_size,
                EXAMPLE_DATA.intermediate.margin_width,
                EXAMPLE_DATA.intermediate.feature_dim,
                EXAMPLE_DATA.intermediate.f0_vector.as_ptr() as *mut f32,
                EXAMPLE_DATA.intermediate.phoneme_vector.as_ptr() as *mut f32,
                &mut { EXAMPLE_DATA.speaker_id } as *mut i64,
                audio_feature.as_mut_ptr(),
            ));
            let full_length = EXAMPLE_DATA.intermediate.f0_length as usize;
            let pitch = EXAMPLE_DATA.intermediate.feature_dim as usize;
            for render_start in (0..full_length).step_by(10) {
                // render_start .. render_end の音声を取得する
                let render_end = min(render_start + 10, full_length);
                let slice_start = render_start;
                let slice_end = render_end + 2 * EXAMPLE_DATA.intermediate.margin_width as usize;
                let feature_segment = &audio_feature[slice_start * pitch..slice_end * pitch];
                let slice_length = slice_end - slice_start;
                let mut wave_segment_with_margin = vec![0.; 256 * slice_length];
                assert!(lib.render_audio_segment(
                    slice_length as i64,
                    pitch as i64,
                    feature_segment.as_ptr() as *mut f32,
                    &mut { EXAMPLE_DATA.speaker_id } as *mut i64,
                    wave_segment_with_margin.as_mut_ptr(),
                ));
                let wave_segment = &wave_segment_with_margin[256
                    * EXAMPLE_DATA.intermediate.margin_width as usize
                    ..wave_segment_with_margin.len()
                        - 256 * EXAMPLE_DATA.intermediate.margin_width as usize];
                wave[render_start * 256..render_end * 256].clone_from_slice(wave_segment);
            }
            wave
        };

        std::assert_eq!(SNAPSHOTS.metas, metas_json);

        float_assert::close_l1(&phoneme_length, &EXAMPLE_DATA.duration.result, 0.01);
        float_assert::close_l1(&intonation_list, &EXAMPLE_DATA.intonation.result, 0.01);

        assert!(wave.iter().copied().all(f32::is_normal));
        assert!(wave2.iter().copied().all(f32::is_normal));
        assert!(wave3.iter().copied().all(f32::is_normal));
        float_assert::close_l1(&wave2, &wave, 0.01);
        float_assert::close_l1(&wave3, &wave, 0.01);

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

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(compatible_engine);

#[derive(Deserialize)]
struct Snapshots {
    metas: String,
    #[serde(deserialize_with = "snapshots::deserialize_platform_specific_snapshot")]
    stderr: String,
}
