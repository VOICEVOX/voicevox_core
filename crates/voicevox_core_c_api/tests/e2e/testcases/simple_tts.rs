use std::{collections::HashMap, ffi::CString, mem::MaybeUninit, sync::LazyLock};

use assert_cmd::assert::AssertResult;
use libloading::Library;
use serde::{Deserialize, Serialize};
use test_util::{
    c_api::{self, CApi, VoicevoxInitializeOptions, VoicevoxResultCode},
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

#[typetag::serde(name = "simple_tts")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        let lib = CApi::from_library(lib)?;

        let model = {
            let mut model = MaybeUninit::uninit();
            assert_ok(lib.voicevox_voice_model_new_from_path(
                c_api::SAMPLE_VOICE_MODEL_FILE_PATH.as_ptr(),
                model.as_mut_ptr(),
            ));
            model.assume_init()
        };

        let onnxruntime = {
            let mut onnxruntime = MaybeUninit::uninit();
            assert_ok(lib.voicevox_onnxruntime_load_once(
                lib.voicevox_make_default_load_onnxruntime_options(),
                onnxruntime.as_mut_ptr(),
            ));
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

        let (wav_length, wav) = {
            let mut wav_length = MaybeUninit::uninit();
            let mut wav = MaybeUninit::uninit();
            let text = CString::new(&*self.text).unwrap();
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

        std::assert_eq!(SNAPSHOTS.output[&self.text].wav_length, wav_length);

        lib.voicevox_voice_model_delete(model);
        lib.voicevox_open_jtalk_rc_delete(openjtalk);
        lib.voicevox_synthesizer_delete(synthesizer);
        lib.voicevox_wav_free(wav);

        return Ok(());

        const STYLE_ID: u32 = 0;

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK, result_code);
        }
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

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(simple_tts);

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
