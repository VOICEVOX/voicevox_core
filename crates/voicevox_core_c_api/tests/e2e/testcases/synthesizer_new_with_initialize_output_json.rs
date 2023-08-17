use std::{
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

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "synthesizer_new_with_initialize_output_json")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: &Library) -> anyhow::Result<()> {
        let Symbols {
            voicevox_make_default_initialize_options,
            voicevox_open_jtalk_rc_new,
            voicevox_open_jtalk_rc_delete,
            voicevox_synthesizer_new_with_initialize,
            voicevox_synthesizer_delete,
            voicevox_synthesizer_create_metas_json,
            voicevox_json_free,
            ..
        } = Symbols::new(lib)?;

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
                    load_all_models: true,
                    ..voicevox_make_default_initialize_options()
                },
                synthesizer.as_mut_ptr(),
            ));
            synthesizer.assume_init()
        };

        let metas_json = {
            let raw = voicevox_synthesizer_create_metas_json(synthesizer);
            let metas_json = &CStr::from_ptr(raw).to_str()?.parse::<serde_json::Value>()?;
            let metas_json = serde_json::to_string_pretty(metas_json).unwrap();
            voicevox_json_free(raw);
            metas_json
        };

        std::assert_eq!(SNAPSHOTS.metas, metas_json);

        voicevox_open_jtalk_rc_delete(openjtalk);
        voicevox_synthesizer_delete(synthesizer);

        return Ok(());

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

static SNAPSHOTS: Lazy<Snapshots> =
    snapshots::section!(synthesizer_new_with_initialize_output_json);

#[derive(Deserialize)]
struct Snapshots {
    metas: String,
    #[serde(deserialize_with = "snapshots::deserialize_platform_specific_snapshot")]
    stderr: String,
}
