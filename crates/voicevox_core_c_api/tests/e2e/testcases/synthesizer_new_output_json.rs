use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
};

use assert_cmd::assert::AssertResult;
use libloading::Library;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use test_util::{
    c_api::{self, CApi, VoicevoxInitializeOptions, VoicevoxResultCode},
    OPEN_JTALK_DIC_DIR,
};

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "synthesizer_new_output_json")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        let lib = CApi::from_library(lib)?;

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

        let model = {
            let mut model = MaybeUninit::uninit();
            assert_ok(lib.voicevox_voice_model_new_from_path(
                c"../../model/sample.vvm".as_ptr(),
                model.as_mut_ptr(),
            ));
            model.assume_init()
        };

        assert_ok(lib.voicevox_synthesizer_load_voice_model(synthesizer, model));

        let metas_json = {
            let raw = lib.voicevox_synthesizer_create_metas_json(synthesizer);
            let metas_json = &CStr::from_ptr(raw).to_str()?.parse::<serde_json::Value>()?;
            let metas_json = serde_json::to_string_pretty(metas_json).unwrap();
            lib.voicevox_json_free(raw);
            metas_json
        };

        std::assert_eq!(SNAPSHOTS.metas, metas_json);

        lib.voicevox_open_jtalk_rc_delete(openjtalk);
        lib.voicevox_synthesizer_delete(synthesizer);

        return Ok(());

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK, result_code);
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

static SNAPSHOTS: Lazy<Snapshots> = snapshots::section!(synthesizer_new_output_json);

#[derive(Deserialize)]
struct Snapshots {
    metas: String,
    #[serde(deserialize_with = "snapshots::deserialize_platform_specific_snapshot")]
    stderr: String,
}
