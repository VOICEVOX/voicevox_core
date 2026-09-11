use std::{
    env,
    ffi::{CStr, CString},
    mem::MaybeUninit,
    sync::LazyLock,
};

use assert_cmd::assert::AssertResult;
use const_format::concatcp;
use libloading::Library;
use serde::{Deserialize, Serialize};
use test_util::{
    OPEN_JTALK_DIC_DIR,
    c_api::{
        self, CApi, VoicevoxInitializeOptions, VoicevoxLoadOnnxruntimeOptions, VoicevoxResultCode,
    },
};

use crate::{
    assert_cdylib::{self, Utf8Output, case},
    snapshots,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "render_empty")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        const TEXT: &CStr = c"こんにちは";

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
                // - `onnxruntime` is valid for reads.
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

        let accent_phrases = {
            let mut accent_phrases = MaybeUninit::uninit();
            assert_ok(unsafe {
                // SAFETY:
                // - `TEXT` is a valid string.
                // - `accent_phrases` is valid for writes.
                lib.voicevox_open_jtalk_rc_analyze(
                    openjtalk,
                    TEXT.as_ptr(),
                    accent_phrases.as_mut_ptr(),
                )
            });
            // SAFETY: `voicevox_open_jtalk_rc_analyze` initializes `accent_phrases` if
            // succeeded.
            unsafe { accent_phrases.assume_init() }
        };
        let accent_phrases = {
            let mut next_accent_phrases = MaybeUninit::uninit();
            assert_ok(unsafe {
                // SAFETY:
                // - `accent_phrases` is a valid string.
                // - `next_accent_phrases` is valid for writes.
                lib.voicevox_synthesizer_replace_phoneme_length(
                    synthesizer,
                    accent_phrases,
                    STYLE_ID,
                    next_accent_phrases.as_mut_ptr(),
                )
            });
            // SAFETY: `accent_phrases` is valid and is no longer used.
            unsafe { lib.voicevox_json_free(accent_phrases) };
            // SAFETY: `voicevox_synthesizer_replace_phoneme_length` initializes
            // `next_accent_phrases` if succeeded.
            unsafe { next_accent_phrases.assume_init() }
        };
        let accent_phrases = {
            let mut next_accent_phrases = MaybeUninit::uninit();
            assert_ok(unsafe {
                // SAFETY:
                // - `accent_phrases` is a valid string.
                // - `next_accent_phrases` is valid for writes.
                lib.voicevox_synthesizer_replace_mora_pitch(
                    synthesizer,
                    accent_phrases,
                    STYLE_ID,
                    next_accent_phrases.as_mut_ptr(),
                )
            });
            // SAFETY: `accent_phrases` is valid and is no longer used.
            unsafe { lib.voicevox_json_free(accent_phrases) };
            // SAFETY: `voicevox_synthesizer_replace_mora_pitch` initializes
            // `next_accent_phrases` if succeeded.
            unsafe { next_accent_phrases.assume_init() }
        };
        let audio_query = {
            let mut audio_query = MaybeUninit::uninit();
            assert_ok(unsafe {
                // SAFETY:
                // - `accent_phrases` is a valid string.
                // - `next_accent_phrases` is valid for writes.
                lib.voicevox_audio_query_create_from_accent_phrases(
                    accent_phrases,
                    audio_query.as_mut_ptr(),
                )
            });
            // SAFETY: `voicevox_audio_query_create_from_accent_phrases` initializes
            // `audio_query` if succeeded.
            unsafe { audio_query.assume_init() }
        };

        let audio_feature = {
            let mut audio_feature = MaybeUninit::uninit();
            assert_ok(unsafe {
                // SAFETY:
                // - `audio_query` is a valid string.
                // - `audio_feature` is valid for writes.
                lib.voicevox_synthesizer_create_audio_feature(
                    synthesizer,
                    audio_query,
                    STYLE_ID,
                    lib.voicevox_make_default_synthesis_options(),
                    audio_feature.as_mut_ptr(),
                )
            });
            // SAFETY: `voicevox_synthesizer_create_audio_feature` initializes `audio_feature`
            // if succeeded.
            unsafe { audio_feature.assume_init() }
        };

        let (pcm_length, pcm) = {
            // SAFETY: this function has no safety requirements.
            let len = unsafe { lib.voicevox_audio_feature_frame_length(audio_feature) };

            let mut pcm_length = MaybeUninit::uninit();
            let mut pcm = MaybeUninit::uninit();
            assert_ok(unsafe {
                // SAFETY: `pcm_length` and `pcm` are valid for writes.
                lib.voicevox_synthesizer_render(
                    synthesizer,
                    audio_feature,
                    len / 2,
                    len / 2,
                    pcm_length.as_mut_ptr(),
                    pcm.as_mut_ptr(),
                )
            });
            // SAFETY: `voicevox_synthesizer_render` initializes `pcm_length` and `pcm` if
            // succeeded.
            unsafe { (pcm_length.assume_init(), pcm.assume_init()) }
        };

        assert_eq!(0, pcm_length);

        // SAFETY: `voicevox_empty_bytes` is a immutable variable.
        assert_eq!(unsafe { *lib.voicevox_empty_bytes() }, pcm);

        // This should emit a warning.
        // SAFETY: no longer used.
        unsafe { lib.voicevox_wav_free(pcm) };

        // SAFETY: they are valid and is no longer used.
        unsafe { lib.voicevox_json_free(accent_phrases) };
        unsafe { lib.voicevox_json_free(audio_query) };

        // SAFETY: these functions have no safety requirements.
        unsafe { lib.voicevox_voice_model_file_delete(model) };
        unsafe { lib.voicevox_open_jtalk_rc_delete(openjtalk) };
        unsafe { lib.voicevox_synthesizer_delete(synthesizer) };
        unsafe { lib.voicevox_audio_feature_delete(audio_feature) };

        return Ok(());

        const STYLE_ID: u32 = 302;

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK, result_code);
        }
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        output
            .mask_timestamps()
            .mask_unix_onnxruntime_filename()
            .mask_windows_video_cards()
            .assert()
            .try_success()?
            .try_stdout("")?
            .try_stderr(&*SNAPSHOTS.stderr)
    }
}

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(render_empty);

#[derive(Deserialize)]
struct Snapshots {
    #[serde(deserialize_with = "snapshots::deserialize_platform_specific_snapshot")]
    stderr: String,
}
