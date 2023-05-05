use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    mem::MaybeUninit,
};

use assert_cmd::assert::AssertResult;
use libloading::Library;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use test_util::OPEN_JTALK_DIC_DIR;
use voicevox_core::result_code::VoicevoxResultCode;

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
    symbols::{Symbols, VoicevoxAccelerationMode, VoicevoxInitializeOptions},
};

macro_rules! cstr {
    ($s:literal $(,)?) => {
        CStr::from_bytes_with_nul(concat!($s, '\0').as_ref()).unwrap()
    };
}

case!(TestCase {
    text: "こんにちは、音声合成の世界へようこそ".to_owned()
});

#[derive(Serialize, Deserialize)]
struct TestCase {
    text: String,
}

#[typetag::serde(name = "tts_via_audio_query")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: &Library) -> anyhow::Result<()> {
        let Symbols {
            voicevox_open_jtalk_rc_new,
            voicevox_open_jtalk_rc_delete,
            voicevox_make_default_initialize_options,
            voicevox_voice_model_new_from_path,
            voicevox_voice_model_delete,
            voicevox_synthesizer_new_with_initialize,
            voicevox_synthesizer_delete,
            voicevox_synthesizer_load_voice_model,
            voicevox_make_default_audio_query_options,
            voicevox_synthesizer_audio_query,
            voicevox_make_default_synthesis_options,
            voicevox_synthesizer_synthesis,
            voicevox_json_free,
            voicevox_wav_free,
            ..
        } = Symbols::new(lib)?;

        let model = {
            let mut model = MaybeUninit::uninit();
            assert_ok(voicevox_voice_model_new_from_path(
                cstr!("../../model/sample.vvm").as_ptr(),
                model.as_mut_ptr(),
            ));
            model.assume_init()
        };

        let openjtalk = {
            let mut openjtalk = MaybeUninit::uninit();
            let open_jtalk_dic_dir = CString::new(OPEN_JTALK_DIC_DIR).unwrap();
            assert_ok(voicevox_open_jtalk_rc_new(
                open_jtalk_dic_dir.as_ptr(),
                openjtalk.as_mut_ptr(),
            ));
            openjtalk.assume_init()
        };

        let synthesizer = {
            let mut synthesizer = MaybeUninit::uninit();
            assert_ok(voicevox_synthesizer_new_with_initialize(
                openjtalk,
                VoicevoxInitializeOptions {
                    acceleration_mode: VoicevoxAccelerationMode::VOICEVOX_ACCELERATION_MODE_CPU,
                    ..voicevox_make_default_initialize_options()
                },
                synthesizer.as_mut_ptr(),
            ));
            synthesizer.assume_init()
        };

        assert_ok(voicevox_synthesizer_load_voice_model(synthesizer, model));

        let audio_query = {
            let mut audio_query = MaybeUninit::uninit();
            let text = CString::new(&*self.text).unwrap();
            assert_ok(voicevox_synthesizer_audio_query(
                synthesizer,
                text.as_ptr(),
                STYLE_ID,
                voicevox_make_default_audio_query_options(),
                audio_query.as_mut_ptr(),
            ));
            audio_query.assume_init()
        };

        let (wav_length, wav) = {
            let mut wav_length = MaybeUninit::uninit();
            let mut wav = MaybeUninit::uninit();
            assert_ok(voicevox_synthesizer_synthesis(
                synthesizer,
                audio_query,
                STYLE_ID,
                voicevox_make_default_synthesis_options(),
                wav_length.as_mut_ptr(),
                wav.as_mut_ptr(),
            ));
            (wav_length.assume_init(), wav.assume_init())
        };

        std::assert_eq!(SNAPSHOTS.output[&self.text].wav_length, wav_length);

        voicevox_voice_model_delete(model);
        voicevox_open_jtalk_rc_delete(openjtalk);
        voicevox_synthesizer_delete(synthesizer);
        voicevox_json_free(audio_query);
        voicevox_wav_free(wav);

        return Ok(());

        const STYLE_ID: u32 = 0;

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(VoicevoxResultCode::VOICEVOX_RESULT_OK, result_code);
        }
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

static SNAPSHOTS: Lazy<Snapshots> = snapshots::section!(tts_via_audio_query);

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
