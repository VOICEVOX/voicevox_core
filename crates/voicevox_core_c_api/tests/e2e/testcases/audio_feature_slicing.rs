use std::{
    env,
    ffi::{CStr, CString},
    fmt::Debug,
    mem::MaybeUninit,
};

use assert_cmd::assert::AssertResult;
use const_format::concatcp;
use itertools::{chain, iproduct};
use libloading::Library;
use serde::{Deserialize, Serialize};
use test_util::{
    OPEN_JTALK_DIC_DIR,
    c_api::{
        self, CApi, VOICEVOX_FRAME_RATE, VoicevoxAudioFeatureSlice, VoicevoxInitializeOptions,
        VoicevoxLoadOnnxruntimeOptions, VoicevoxResultCode,
        VoicevoxResultCode_VOICEVOX_RESULT_END_OUT_OF_BOUNDS_ERROR as END_OUT_OF_BOUNDS,
        VoicevoxResultCode_VOICEVOX_RESULT_OK as OK,
        VoicevoxResultCode_VOICEVOX_RESULT_START_EXCEEDS_END_ERROR as START_EXCEEDS_END,
        VoicevoxResultCode_VOICEVOX_RESULT_START_OUT_OF_BOUNDS_ERROR as START_OUT_OF_BOUNDS,
    },
};

use crate::assert_cdylib::{self, Utf8Output, case};

const LEN: usize = 10;

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "audio_feature_slicing")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        const STYLE_ID: u32 = 302;

        // Avoid logging every expected bounds error in the exhaustive checks below.
        // SAFETY: This case runs in its own subprocess, which is still single-threaded.
        // No C API calls or ONNX Runtime initialization have taken place yet.
        unsafe { env::set_var("RUST_LOG", "off") };

        // SAFETY: The safety contract must be upheld by the caller.
        let lib = unsafe { CApi::from_library(lib) }?;

        let model = {
            let mut model = MaybeUninit::uninit();
            assert_ok(unsafe {
                // SAFETY:
                // - `SAMPLE_VOICE_MODEL_FILE_PATH` is a valid string.
                // - `model` is valid for writes.
                lib.voicevox_voice_model_file_open(
                    c_api::SAMPLE_VOICE_MODEL_FILE_PATH.as_ptr(),
                    model.as_mut_ptr(),
                )
            });
            // SAFETY: `voicevox_voice_model_file_open` initializes `model` if succeeded.
            unsafe { model.assume_init() }
        };

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

        let openjtalk = {
            let mut openjtalk = MaybeUninit::uninit();
            let open_jtalk_dic_dir = CString::new(OPEN_JTALK_DIC_DIR).unwrap();
            assert_ok(unsafe {
                // SAFETY:
                // - A `CString` is a valid string.
                // - `openjtalk` is valid for writes.
                lib.voicevox_open_jtalk_rc_new(open_jtalk_dic_dir.as_ptr(), openjtalk.as_mut_ptr())
            });
            // SAFETY: `voicevox_open_jtalk_rc_new` initializes `openjtalk` if succeeded.
            unsafe { openjtalk.assume_init() }
        };

        let synthesizer = {
            let mut synthesizer = MaybeUninit::uninit();
            assert_ok(unsafe {
                // SAFETY:
                // - `onnxruntime` and `openjtalk` are valid.
                // - `synthesizer` is valid for writes.
                lib.voicevox_synthesizer_new(
                    onnxruntime,
                    openjtalk,
                    VoicevoxInitializeOptions {
                        acceleration_mode:
                            c_api::VoicevoxAccelerationMode_VOICEVOX_ACCELERATION_MODE_CPU,
                        ..lib.voicevox_make_default_initialize_options()
                    },
                    synthesizer.as_mut_ptr(),
                )
            });
            // SAFETY: `voicevox_synthesizer_new` initializes `synthesizer` if succeeded.
            unsafe { synthesizer.assume_init() }
        };

        // SAFETY: `voicevox_synthesizer_load_voice_model` has no safety requirements.
        assert_ok(unsafe {
            lib.voicevox_synthesizer_load_voice_model(
                synthesizer,
                model,
                lib.voicevox_make_default_load_voice_model_options(),
            )
        });

        let audio_query = CString::new(
            serde_json::json!({
                "accent_phrases": [],
                "speedScale": 1.0,
                "pitchScale": 0.0,
                "intonationScale": 1.0,
                "volumeScale": 1.0,
                "prePhonemeLength": LEN as f64 / VOICEVOX_FRAME_RATE,
                "postPhonemeLength": 0.0,
                "outputSamplingRate": 24000,
                "outputStereo": false,
            })
            .to_string(),
        )?;
        let audio_feature = {
            let mut audio_feature = MaybeUninit::uninit();
            assert_ok(unsafe {
                // SAFETY:
                // - `audio_query` is a valid string.
                // - `audio_feature` is valid for writes.
                lib.voicevox_synthesizer_create_audio_feature(
                    synthesizer,
                    audio_query.as_ptr(),
                    STYLE_ID,
                    lib.voicevox_make_default_synthesis_options(),
                    audio_feature.as_mut_ptr(),
                )
            });
            // SAFETY: `voicevox_synthesizer_create_audio_feature` initializes `audio_feature`
            // if succeeded.
            unsafe { audio_feature.assume_init() }
        };

        // SAFETY: `audio_feature` is valid.
        assert_eq!(LEN, unsafe {
            lib.voicevox_audio_feature_len(audio_feature)
        });
        // SAFETY: `audio_feature` is valid.
        assert!(!unsafe { lib.voicevox_audio_feature_is_empty(audio_feature) });

        let check_slice = |slice: VoicevoxAudioFeatureSlice,
                           (offset, len): (usize, usize),
                           context: &dyn Debug| {
            assert_eq!(
                audio_feature.cast_const(),
                slice.audio_feature,
                "{context:?}"
            );
            assert_eq!(offset, slice.start_offset, "{context:?}");

            let mut actual_len = MaybeUninit::uninit();
            let mut actual_is_empty = MaybeUninit::uninit();
            // SAFETY: `audio_feature` is still alive and `actual_len` is valid for writes.
            assert_eq!(
                OK,
                unsafe { lib.voicevox_audio_feature_slice_len(slice, actual_len.as_mut_ptr()) },
                "{context:?}",
            );
            // SAFETY: `audio_feature` is still alive and `actual_is_empty` is valid for writes.
            assert_eq!(
                OK,
                unsafe {
                    lib.voicevox_audio_feature_slice_is_empty(slice, actual_is_empty.as_mut_ptr())
                },
                "{context:?}",
            );
            // SAFETY: Both outputs were initialized by successful calls above.
            let (actual_len, actual_is_empty) =
                unsafe { (actual_len.assume_init(), actual_is_empty.assume_init()) };
            assert_eq!(len, actual_len, "{context:?}");
            assert_eq!(len == 0, actual_is_empty, "{context:?}");
        };

        let check_result = |code: VoicevoxResultCode,
                            output: MaybeUninit<VoicevoxAudioFeatureSlice>,
                            expected: Result<(usize, usize), VoicevoxResultCode>,
                            context: &dyn Debug| {
            let expected_code = match expected {
                Ok(_) => OK,
                Err(code) => code,
            };
            assert_eq!(expected_code, code, "{context:?}");
            expected.ok().map(|expected| {
                // SAFETY: The successful slicing call initialized `output`.
                let slice = unsafe { output.assume_init() };
                check_slice(slice, expected, context);
                slice
            })
        };

        for (clamp, allow_start_exceeds_end) in iproduct!([false, true], [false, true]) {
            // SAFETY: `audio_feature` is valid.
            let slice = unsafe {
                lib.voicevox_audio_feature_slice_new(audio_feature, clamp, allow_start_exceeds_end)
            };
            check_slice(
                slice,
                (0, LEN),
                &("slice_new", clamp, allow_start_exceeds_end),
            );
        }

        let reference = [(); LEN];
        for outer in SliceArgs::all() {
            let expected = outer.reference_slice(&reference);
            let mut output = MaybeUninit::uninit();
            // SAFETY: `audio_feature` is valid and `output` is valid for writes.
            let code = unsafe {
                lib.voicevox_audio_feature_slice(
                    audio_feature,
                    outer.start,
                    outer.end,
                    outer.end_is_inclusive,
                    outer.clamp,
                    outer.allow_start_exceeds_end,
                    output.as_mut_ptr(),
                )
            };
            let Some(slice) = check_result(
                code,
                output,
                expected.map(|(offset, slice)| (offset, slice.len())),
                &outer,
            ) else {
                continue;
            };
            let (offset, reference) = expected.expect("checked the successful result above");

            for inner in SliceArgs::all() {
                let expected = inner
                    .reference_slice(reference)
                    .map(|(start, slice)| (offset + start, slice.len()));
                let mut output = MaybeUninit::uninit();
                // SAFETY: `audio_feature` is still alive and `output` is valid for writes.
                let code = unsafe {
                    lib.voicevox_audio_feature_slice_slice(
                        slice,
                        inner.start,
                        inner.end,
                        inner.end_is_inclusive,
                        inner.clamp,
                        inner.allow_start_exceeds_end,
                        output.as_mut_ptr(),
                    )
                };
                check_result(code, output, expected, &(outer, inner));
            }
        }

        // SAFETY: These functions have no safety requirements. The objects are no longer used.
        unsafe {
            lib.voicevox_audio_feature_delete(audio_feature);
            lib.voicevox_synthesizer_delete(synthesizer);
            lib.voicevox_open_jtalk_rc_delete(openjtalk);
            lib.voicevox_voice_model_file_delete(model);
        }

        return Ok(());

        fn assert_ok(result_code: VoicevoxResultCode) {
            assert_eq!(OK, result_code);
        }
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        output
            .assert()
            .try_success()?
            .try_stdout("")?
            .try_stderr("")
    }
}

#[derive(Clone, Copy, Debug)]
struct SliceArgs {
    start: usize,
    end: usize,
    end_is_inclusive: bool,
    clamp: bool,
    allow_start_exceeds_end: bool,
}

impl SliceArgs {
    fn all() -> impl Iterator<Item = Self> {
        let bounds = chain!(0..=LEN + 1, [usize::MAX]);
        iproduct!(
            bounds.clone(),
            bounds,
            [false, true],
            [false, true],
            [false, true]
        )
        .map(
            |(start, end, end_is_inclusive, clamp, allow_start_exceeds_end)| Self {
                start,
                end,
                end_is_inclusive,
                clamp,
                allow_start_exceeds_end,
            },
        )
    }

    fn reference_slice(self, reference: &[()]) -> Result<(usize, &[()]), VoicevoxResultCode> {
        let Self {
            mut start,
            mut end,
            end_is_inclusive,
            clamp,
            allow_start_exceeds_end,
        } = self;

        if clamp {
            let max = if end_is_inclusive {
                reference.len().checked_sub(1).ok_or(END_OUT_OF_BOUNDS)?
            } else {
                reference.len()
            };
            start = start.min(max);
            end = end.min(max);
        }

        let slice = if end_is_inclusive {
            reference.get(start..=end)
        } else {
            reference.get(start..end)
        };
        if let Some(slice) = slice {
            Ok((start, slice))
        } else if start > reference.len() {
            Err(START_OUT_OF_BOUNDS)
        } else if end > reference.len() || (end_is_inclusive && end == reference.len()) {
            Err(END_OUT_OF_BOUNDS)
        } else if allow_start_exceeds_end {
            Ok((start, &reference[start..start]))
        } else {
            Err(START_EXCEEDS_END)
        }
    }
}
