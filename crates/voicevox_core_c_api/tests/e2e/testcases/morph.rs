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
use test_util::{
    c_api::{self, CApi, VoicevoxInitializeOptions, VoicevoxResultCode, VoicevoxStyleId},
    OPEN_JTALK_DIC_DIR,
};

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
};

const TEXT: &CStr = cstr!("こんにちは、音声合成の世界へようこそ");
const MORPH_RATE: f64 = 0.5;

case!(TestCase {
    base_style: 0,
    target_style: 0,
});
case!(TestCase {
    base_style: 0,
    target_style: 1,
});
case!(TestCase {
    base_style: 0,
    target_style: 302,
});
case!(TestCase {
    base_style: 0,
    target_style: 303,
});

case!(TestCase {
    base_style: 1,
    target_style: 0,
});
case!(TestCase {
    base_style: 1,
    target_style: 1,
});
case!(TestCase {
    base_style: 1,
    target_style: 302,
});
case!(TestCase {
    base_style: 1,
    target_style: 303,
});

case!(TestCase {
    base_style: 302,
    target_style: 0,
});
case!(TestCase {
    base_style: 302,
    target_style: 1,
});
case!(TestCase {
    base_style: 302,
    target_style: 302,
});
case!(TestCase {
    base_style: 302,
    target_style: 303,
});

case!(TestCase {
    base_style: 303,
    target_style: 0,
});
case!(TestCase {
    base_style: 303,
    target_style: 1,
});
case!(TestCase {
    base_style: 303,
    target_style: 302,
});
case!(TestCase {
    base_style: 303,
    target_style: 303,
});

#[derive(Serialize, Deserialize)]
struct TestCase {
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
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        let lib = CApi::from_library(lib)?;

        let model = {
            let mut model = MaybeUninit::uninit();
            assert_ok(lib.voicevox_voice_model_new_from_path(
                cstr!("../../model/sample.vvm").as_ptr(),
                model.as_mut_ptr(),
            ));
            model.assume_init()
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

        let audio_query = {
            let mut audio_query = MaybeUninit::uninit();
            assert_ok(lib.voicevox_synthesizer_create_audio_query(
                synthesizer,
                TEXT.as_ptr(),
                self.base_style,
                audio_query.as_mut_ptr(),
            ));
            audio_query.assume_init()
        };

        let morphable_targets = {
            let mut morphable_target = MaybeUninit::uninit();
            assert_ok(lib.voicevox_synthesizer_create_morphable_targets_json(
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

        // TODO: スナップショットテストをやる
        let result = {
            let mut wav_length = MaybeUninit::uninit();
            let mut wav = MaybeUninit::uninit();
            let result = lib.voicevox_synthesizer_synthesis_morphing(
                synthesizer,
                audio_query,
                self.base_style,
                self.target_style,
                MORPH_RATE,
                wav_length.as_mut_ptr(),
                wav.as_mut_ptr(),
            );
            match result {
                c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK => Ok(wav.assume_init()),
                c_api::VoicevoxResultCode_VOICEVOX_RESULT_SPEAKER_FEATURE_ERROR => Err(()),
                result => bail!("code = {result:?}"),
            }
        };

        std::assert_eq!(is_morphable, result.is_ok());
        std::assert_eq!(SNAPSHOTS[&self.to_string()].ok, result.is_ok());

        lib.voicevox_voice_model_delete(model);
        lib.voicevox_open_jtalk_rc_delete(openjtalk);
        lib.voicevox_synthesizer_delete(synthesizer);
        lib.voicevox_json_free(audio_query);
        lib.voicevox_json_free(morphable_targets);
        if let Ok(wav) = result {
            lib.voicevox_wav_free(wav);
        }
        return Ok(());

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK, result_code);
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
