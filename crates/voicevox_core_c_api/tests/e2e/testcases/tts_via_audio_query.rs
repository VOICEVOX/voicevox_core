use std::{
    collections::HashMap,
    env,
    ffi::{CStr, CString},
    mem::MaybeUninit,
    slice,
    sync::LazyLock,
};

use assert_cmd::assert::AssertResult;
use const_format::concatcp;
use libloading::Library;
use serde::{Deserialize, Serialize};
use test_util::{
    c_api::{
        self, CApi, VoicevoxInitializeOptions, VoicevoxLoadOnnxruntimeOptions, VoicevoxResultCode,
    },
    OPEN_JTALK_DIC_DIR,
};

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
};

case!(TestCase {
    text: "こんにちは、音声合成の世界へようこそ".to_owned()
});

#[derive(Serialize, Deserialize)]
struct TestCase {
    text: String,
}

#[typetag::serde(name = "tts_via_audio_query")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        let lib = CApi::from_library(lib)?;

        let model = {
            let mut model = MaybeUninit::uninit();
            assert_ok(lib.voicevox_voice_model_file_open(
                c_api::SAMPLE_VOICE_MODEL_FILE_PATH.as_ptr(),
                model.as_mut_ptr(),
            ));
            model.assume_init()
        };

        let onnxruntime = {
            let mut onnxruntime = MaybeUninit::uninit();
            assert_ok(
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
                ),
            );
            onnxruntime.assume_init()
        };

        let openjtalk = {
            let mut openjtalk = MaybeUninit::uninit();
            let open_jtalk_dic_dir = CString::new(OPEN_JTALK_DIC_DIR).unwrap();
            assert_ok(
                lib.voicevox_open_jtalk_rc_new(open_jtalk_dic_dir.as_ptr(), openjtalk.as_mut_ptr()),
            );
            openjtalk.assume_init()
        };

        let synthesizer = {
            let mut synthesizer = MaybeUninit::uninit();
            assert_ok(lib.voicevox_synthesizer_new(
                onnxruntime,
                openjtalk,
                VoicevoxInitializeOptions {
                    acceleration_mode:
                        c_api::VoicevoxAccelerationMode_VOICEVOX_ACCELERATION_MODE_CPU,
                    ..lib.voicevox_make_default_initialize_options()
                },
                synthesizer.as_mut_ptr(),
            ));
            synthesizer.assume_init()
        };

        assert_ok(lib.voicevox_synthesizer_load_voice_model(synthesizer, model));

        let text = CString::new(&*self.text).unwrap();

        // `voicevox_synthesizer_tts`
        let (wav_length1, wav1) = {
            let mut wav_length = MaybeUninit::uninit();
            let mut wav = MaybeUninit::uninit();
            assert_ok(lib.voicevox_synthesizer_tts(
                synthesizer,
                text.as_ptr(),
                STYLE_ID,
                lib.voicevox_make_default_tts_options(),
                wav_length.as_mut_ptr(),
                wav.as_mut_ptr(),
            ));
            (wav_length.assume_init(), wav.assume_init())
        };

        // `voicevox_synthesizer_create_audio_query`
        // → `voicevox_synthesizer_synthesis`
        let (wav_length2, wav2) = {
            let audio_query = {
                let mut audio_query = MaybeUninit::uninit();
                assert_ok(lib.voicevox_synthesizer_create_audio_query(
                    synthesizer,
                    text.as_ptr(),
                    STYLE_ID,
                    audio_query.as_mut_ptr(),
                ));
                audio_query.assume_init()
            };

            let mut wav_length = MaybeUninit::uninit();
            let mut wav = MaybeUninit::uninit();
            assert_ok(lib.voicevox_synthesizer_synthesis(
                synthesizer,
                audio_query,
                STYLE_ID,
                lib.voicevox_make_default_synthesis_options(),
                wav_length.as_mut_ptr(),
                wav.as_mut_ptr(),
            ));
            lib.voicevox_json_free(audio_query);
            (wav_length.assume_init(), wav.assume_init())
        };

        // `voicevox_open_jtalk_rc_analyze`
        // → `voicevox_synthesizer_replace_mora_data`
        // → `voicevox_audio_query_create_from_accent_phrases`
        // → `voicevox_synthesizer_synthesis`
        let (wav_length3, wav3) = {
            let accent_phrases = {
                let mut accent_phrases = MaybeUninit::uninit();
                assert_ok(lib.voicevox_open_jtalk_rc_analyze(
                    openjtalk,
                    text.as_ptr(),
                    accent_phrases.as_mut_ptr(),
                ));
                accent_phrases.assume_init()
            };
            let accent_phrases = {
                let mut next_accent_phrases = MaybeUninit::uninit();
                assert_ok(lib.voicevox_synthesizer_replace_mora_data(
                    synthesizer,
                    accent_phrases,
                    STYLE_ID,
                    next_accent_phrases.as_mut_ptr(),
                ));
                lib.voicevox_json_free(accent_phrases);
                next_accent_phrases.assume_init()
            };
            let audio_query = {
                let mut audio_query = MaybeUninit::uninit();
                assert_ok(lib.voicevox_audio_query_create_from_accent_phrases(
                    accent_phrases,
                    audio_query.as_mut_ptr(),
                ));
                lib.voicevox_json_free(accent_phrases);
                audio_query.assume_init()
            };

            let mut wav_length = MaybeUninit::uninit();
            let mut wav = MaybeUninit::uninit();
            assert_ok(lib.voicevox_synthesizer_synthesis(
                synthesizer,
                audio_query,
                STYLE_ID,
                lib.voicevox_make_default_synthesis_options(),
                wav_length.as_mut_ptr(),
                wav.as_mut_ptr(),
            ));
            lib.voicevox_json_free(audio_query);
            (wav_length.assume_init(), wav.assume_init())
        };

        // `voicevox_open_jtalk_rc_analyze`
        // → `voicevox_synthesizer_replace_phoneme_length`
        // → `voicevox_synthesizer_replace_mora_pitch`
        // → `voicevox_audio_query_create_from_accent_phrases`
        // → `voicevox_synthesizer_synthesis`
        let (wav_length4, wav4) = {
            let accent_phrases = {
                let mut accent_phrases = MaybeUninit::uninit();
                assert_ok(lib.voicevox_open_jtalk_rc_analyze(
                    openjtalk,
                    text.as_ptr(),
                    accent_phrases.as_mut_ptr(),
                ));
                accent_phrases.assume_init()
            };
            let accent_phrases = {
                let mut next_accent_phrases = MaybeUninit::uninit();
                assert_ok(lib.voicevox_synthesizer_replace_phoneme_length(
                    synthesizer,
                    accent_phrases,
                    STYLE_ID,
                    next_accent_phrases.as_mut_ptr(),
                ));
                lib.voicevox_json_free(accent_phrases);
                next_accent_phrases.assume_init()
            };
            let accent_phrases = {
                let mut next_accent_phrases = MaybeUninit::uninit();
                assert_ok(lib.voicevox_synthesizer_replace_mora_pitch(
                    synthesizer,
                    accent_phrases,
                    STYLE_ID,
                    next_accent_phrases.as_mut_ptr(),
                ));
                lib.voicevox_json_free(accent_phrases);
                next_accent_phrases.assume_init()
            };
            let audio_query = {
                let mut audio_query = MaybeUninit::uninit();
                assert_ok(lib.voicevox_audio_query_create_from_accent_phrases(
                    accent_phrases,
                    audio_query.as_mut_ptr(),
                ));
                lib.voicevox_json_free(accent_phrases);
                audio_query.assume_init()
            };

            let mut wav_length = MaybeUninit::uninit();
            let mut wav = MaybeUninit::uninit();
            assert_ok(lib.voicevox_synthesizer_synthesis(
                synthesizer,
                audio_query,
                STYLE_ID,
                lib.voicevox_make_default_synthesis_options(),
                wav_length.as_mut_ptr(),
                wav.as_mut_ptr(),
            ));
            lib.voicevox_json_free(audio_query);
            (wav_length.assume_init(), wav.assume_init())
        };

        std::assert_eq!(SNAPSHOTS.output[&self.text].wav_length, wav_length1);

        std::assert_eq!(
            unsafe { slice::from_raw_parts(wav2, wav_length2) },
            unsafe { slice::from_raw_parts(wav1, wav_length1) },
        );
        std::assert_eq!(
            unsafe { slice::from_raw_parts(wav3, wav_length3) },
            unsafe { slice::from_raw_parts(wav1, wav_length1) },
        );
        std::assert_eq!(
            unsafe { slice::from_raw_parts(wav4, wav_length4) },
            unsafe { slice::from_raw_parts(wav1, wav_length1) },
        );

        lib.voicevox_voice_model_file_delete(model);
        lib.voicevox_open_jtalk_rc_delete(openjtalk);
        lib.voicevox_synthesizer_delete(synthesizer);
        lib.voicevox_wav_free(wav1);
        lib.voicevox_wav_free(wav2);
        lib.voicevox_wav_free(wav3);
        lib.voicevox_wav_free(wav4);

        return Ok(());

        const STYLE_ID: u32 = 0;

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK, result_code);
        }
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        output
            .mask_timestamps()
            .mask_onnxruntime_filename()
            .mask_windows_video_cards()
            .assert()
            .try_success()?
            .try_stdout("")?
            .try_stderr(&*SNAPSHOTS.stderr)
    }
}

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(tts_via_audio_query);

#[derive(Deserialize)]
struct Snapshots {
    output: HashMap<String, ExpectedOutput>,
    #[serde(deserialize_with = "snapshots::deserialize_platform_specific_snapshot")]
    stderr: String,
}

#[derive(Deserialize)]
struct ExpectedOutput {
    wav_length: usize,
}
