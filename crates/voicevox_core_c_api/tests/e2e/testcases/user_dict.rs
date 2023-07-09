// エンジンを起動してyukarin_s・yukarin_sa・decodeの推論を行う

use assert_cmd::assert::AssertResult;
use once_cell::sync::Lazy;
use std::ffi::CStr;
use tempfile::NamedTempFile;
use voicevox_core::result_code::VoicevoxResultCode;

use libloading::Library;
use serde::{Deserialize, Serialize};

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
    symbols::Symbols,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "user_dict")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: &Library) -> anyhow::Result<()> {
        let Symbols {
            voicevox_default_user_dict_word,
            voicevox_dict_new,
            voicevox_dict_add_word,
            voicevox_dict_alter_word,
            voicevox_dict_remove_word,
            voicevox_dict_get_words_json,
            voicevox_dict_merge,
            ..
        } = Symbols::new(lib)?;

        let mut dict = std::ptr::null_mut();

        let temp_dict_path = NamedTempFile::new()?.into_temp_path();
        assert_ok(voicevox_dict_new(
            temp_dict_path.to_str().unwrap().as_ptr() as *const i8,
            &mut dict,
        ));

        let word = voicevox_default_user_dict_word();
        let mut word_uuid = std::ptr::null_mut();

        assert_ok(voicevox_dict_add_word(dict, &word, &mut word_uuid));

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

static SNAPSHOTS: Lazy<Snapshots> = snapshots::section!(global_info);

#[derive(Deserialize)]
struct Snapshots {
    stderr: String,
}
