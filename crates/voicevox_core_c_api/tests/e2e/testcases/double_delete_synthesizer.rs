//! `voicevox_synthesizer_delete`を二度呼ぶとクラッシュすることを確認する。

use std::{ffi::CString, mem::MaybeUninit, sync::LazyLock};

use assert_cmd::assert::AssertResult;
use indexmap::IndexSet;
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

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "double_delete_synthesizer")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        let lib = CApi::from_library(lib)?;

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

        lib.voicevox_synthesizer_delete(synthesizer);
        lib.voicevox_synthesizer_delete(synthesizer);
        unreachable!();

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK, result_code);
        }
    }

    fn assert_output(&self, output: Utf8Output) -> AssertResult {
        let mut assert = output.assert().try_failure()?.try_stdout("")?;
        for s in &SNAPSHOTS.stderr_contains_all {
            assert = assert.try_stderr(predicates::str::contains(s))?;
        }
        Ok(assert)
    }
}

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(double_delete_synthesizer);

#[derive(Deserialize)]
struct Snapshots {
    stderr_contains_all: IndexSet<String>,
}
