use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    fmt::{self, Display},
    mem::MaybeUninit,
};

use anyhow::bail;
use assert_cmd::assert::AssertResult;
use cstr::cstr;
use libloading::Library;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use test_util::OPEN_JTALK_DIC_DIR;

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
    symbols::{
        Symbols, VoicevoxAccelerationMode, VoicevoxInitializeOptions, VoicevoxResultCode,
        VoicevoxStyleId,
    },
};

case!(TestCase {
    text: "こんにちは、音声合成の世界へようこそ".to_owned(),
    base_style: 1,
    target_style: 1,
});
case!(TestCase {
    text: "こんにちは、音声合成の世界へようこそ".to_owned(),
    base_style: 302,
    target_style: 303,
});
case!(TestCase {
    text: "こんにちは、音声合成の世界へようこそ".to_owned(),
    base_style: 1,
    target_style: 302,
});

#[derive(Serialize, Deserialize)]
struct TestCase {
    text: String,
    base_style: VoicevoxStyleId,
    target_style: VoicevoxStyleId,
}

impl Display for TestCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&serde_json::to_string(self).unwrap())
    }
}

#[typetag::serde(name = "morph")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: &Library) -> anyhow::Result<()> {
        let Symbols {
            voicevox_open_jtalk_rc_new,
            voicevox_open_jtalk_rc_delete,
            voicevox_make_default_initialize_options,
            voicevox_voice_model_new_from_path,
            voicevox_voice_model_delete,
            voicevox_synthesizer_new,
            voicevox_synthesizer_delete,
            voicevox_synthesizer_load_voice_model,
            voicevox_synthesizer_create_morphable_targets_json,
            voicevox_synthesizer_create_audio_query,
            voicevox_synthesizer_synthesis_morphing,
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
            assert_ok(voicevox_synthesizer_new(
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
            assert_ok(voicevox_synthesizer_create_audio_query(
                synthesizer,
                text.as_ptr(),
                self.base_style,
                audio_query.as_mut_ptr(),
            ));
            audio_query.assume_init()
        };

        let morphable_targets = {
            let mut morphable_target = MaybeUninit::uninit();
            assert_ok(voicevox_synthesizer_create_morphable_targets_json(
                synthesizer,
                self.base_style,
                morphable_target.as_mut_ptr(),
            ));
            morphable_target.assume_init()
        };

        let MorphableTargetInfo { is_morphable } =
            serde_json::from_slice::<HashMap<VoicevoxStyleId, MorphableTargetInfo>>(
                CStr::from_ptr(morphable_targets).to_bytes(),
            )?[&self.target_style];

        let result = {
            const MORPH_RATE: f32 = 0.5;

            let mut wav_length = MaybeUninit::uninit();
            let mut wav = MaybeUninit::uninit();
            let result = voicevox_synthesizer_synthesis_morphing(
                synthesizer,
                audio_query,
                self.base_style,
                self.target_style,
                MORPH_RATE,
                wav_length.as_mut_ptr(),
                wav.as_mut_ptr(),
            );
            match result {
                VoicevoxResultCode::VOICEVOX_RESULT_OK => Ok(wav.assume_init()),
                VoicevoxResultCode::VOICEVOX_RESULT_SPEAKER_FEATURE => Err(()),
                result => bail!("code = {result:?}"),
            }
        };

        std::assert_eq!(is_morphable, result.is_ok());
        std::assert_eq!(SNAPSHOTS[&self.to_string()].ok, result.is_ok());

        voicevox_voice_model_delete(model);
        voicevox_open_jtalk_rc_delete(openjtalk);
        voicevox_synthesizer_delete(synthesizer);
        voicevox_json_free(audio_query);
        voicevox_json_free(morphable_targets);
        if let Ok(wav) = result {
            voicevox_wav_free(wav);
        }
        return Ok(());

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(VoicevoxResultCode::VOICEVOX_RESULT_OK, result_code);
        }

        #[derive(Deserialize)]
        struct MorphableTargetInfo {
            is_morphable: bool,
        }
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        output
            .mask_timestamps()
            .mask_windows_video_cards()
            .assert()
            .try_success()?
            .try_stdout("")?
            .try_stderr(&*SNAPSHOTS[&self.to_string()].stderr)
    }
}

static SNAPSHOTS: Lazy<HashMap<String, Snapshot>> = snapshots::section!(morph);

#[derive(Deserialize)]
struct Snapshot {
    ok: bool,
    #[serde(deserialize_with = "snapshots::deserialize_platform_specific_snapshot")]
    stderr: String,
}
