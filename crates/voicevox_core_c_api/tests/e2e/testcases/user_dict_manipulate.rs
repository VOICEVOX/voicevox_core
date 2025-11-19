// ユーザー辞書の操作をテストする。

use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    sync::LazyLock,
};

use assert_cmd::assert::AssertResult;
use tempfile::NamedTempFile;
use uuid::Uuid;

use libloading::Library;
use serde::{Deserialize, Serialize};
use test_util::c_api::{self, CApi, VoicevoxResultCode, VoicevoxUserDict, VoicevoxUserDictWord};

use crate::{
    assert_cdylib::{self, Utf8Output, case},
    snapshots,
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "user_dict_manipulate")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: Library) -> anyhow::Result<()> {
        // SAFETY: The safety contract must be upheld by the caller.
        let lib = unsafe { CApi::from_library(lib) }?;

        let get_json = |dict: *const VoicevoxUserDict| -> String {
            let mut json = MaybeUninit::uninit();

            // SAFETY: `json` is valid for writes.
            assert_ok(unsafe { lib.voicevox_user_dict_to_json(dict, json.as_mut_ptr()) });

            // SAFETY: `voicevox_user_dict_to_json` initializes `json` if succeeded.
            let ret = unsafe { CStr::from_ptr(json.assume_init()) }
                .to_str()
                .unwrap()
                .to_string();

            // SAFETY: `json` is valid and is no longer used.
            unsafe { lib.voicevox_json_free(json.assume_init()) };

            serde_json::from_str::<serde_json::Value>(&ret).expect("invalid json");

            ret
        };

        // FIXME: This closure itself should be `unsafe`.
        let add_word = |dict: *const VoicevoxUserDict, word: &VoicevoxUserDictWord| -> Uuid {
            let mut word_uuid = [0u8; _];
            assert_ok(unsafe {
                // SAFETY:
                // - `dict.surface` and `dict.pronunciation` are valid.
                // - `word_uuid` is valid for writes.
                lib.voicevox_user_dict_add_word(dict, &raw const *word, &mut word_uuid)
            });
            Uuid::from_slice(&word_uuid).expect("invalid uuid")
        };

        // テスト用の辞書ファイルを作成
        // SAFETY: `voicevox_user_dict_new` has no safety requirements.
        let dict = unsafe { lib.voicevox_user_dict_new() };

        // 単語の追加のテスト
        // SAFETY: `voicevox_user_dict_word_make` itself has no safety requirements.
        let word =
            unsafe { lib.voicevox_user_dict_word_make(c"hoge".as_ptr(), c"ホゲ".as_ptr(), 0) };

        let word_uuid = add_word(dict, &word);

        let json = get_json(dict);

        assert!(json.contains("ｈｏｇｅ"));
        assert!(json.contains("ホゲ"));
        assert_contains_uuid(&json, &word_uuid);

        // 単語の変更のテスト
        // SAFETY: `voicevox_user_dict_word_make` itself has no safety requirements.
        let word =
            unsafe { lib.voicevox_user_dict_word_make(c"fuga".as_ptr(), c"フガ".as_ptr(), 0) };

        assert_ok(unsafe {
            // SAFETY:
            // - A `[u8; 16]` is valid for reads.
            // - `dict.surface` and `dict.pronunciation` are valid.
            lib.voicevox_user_dict_update_word(dict, &word_uuid.into_bytes(), &word)
        });

        let json = get_json(dict);

        assert!(!json.contains("ｈｏｇｅ"));
        assert!(!json.contains("ホゲ"));
        assert!(json.contains("ｆｕｇａ"));
        assert!(json.contains("フガ"));
        assert_contains_uuid(&json, &word_uuid);

        // 辞書のインポートのテスト。
        // SAFETY: `voicevox_user_dict_new` has no safety requirements.
        let other_dict = unsafe { lib.voicevox_user_dict_new() };

        // SAFETY: `voicevox_user_dict_word_make` itself has no safety requirements.
        let other_word =
            unsafe { lib.voicevox_user_dict_word_make(c"piyo".as_ptr(), c"ピヨ".as_ptr(), 0) };

        let other_word_uuid = add_word(other_dict, &other_word);

        // SAFETY: `voicevox_user_dict_import` has no safety requirements.
        assert_ok(unsafe { lib.voicevox_user_dict_import(dict, other_dict) });

        let json = get_json(dict);
        assert!(json.contains("ｆｕｇａ"));
        assert!(json.contains("フガ"));
        assert_contains_uuid(&json, &word_uuid);
        assert!(json.contains("ｐｉｙｏ"));
        assert!(json.contains("ピヨ"));
        assert_contains_uuid(&json, &other_word_uuid);

        // 単語の削除のテスト
        // SAFETY: A `[u8; 16]` is valid for reads.
        assert_ok(unsafe { lib.voicevox_user_dict_remove_word(dict, &word_uuid.into_bytes()) });

        let json = get_json(dict);
        assert_not_contains_uuid(&json, &word_uuid);
        // 他の単語は残っている
        assert_contains_uuid(&json, &other_word_uuid);

        // 辞書のセーブ・ロードのテスト
        let temp_path = NamedTempFile::new().unwrap().into_temp_path();
        let temp_path = CString::new(temp_path.to_str().unwrap()).unwrap();
        // SAFETY: `voicevox_user_dict_word_make` itself has no safety requirements.
        let word =
            unsafe { lib.voicevox_user_dict_word_make(c"hoge".as_ptr(), c"ホゲ".as_ptr(), 0) };
        let word_uuid = add_word(dict, &word);

        // SAFETY: A `CString` is a valid string.
        assert_ok(unsafe { lib.voicevox_user_dict_save(dict, temp_path.as_ptr()) });
        assert_ok(unsafe { lib.voicevox_user_dict_load(other_dict, temp_path.as_ptr()) });

        let json = get_json(other_dict);
        assert_contains_uuid(&json, &word_uuid);
        assert_contains_uuid(&json, &other_word_uuid);

        // SAFETY: `voicevox_user_dict_delete` has no safety requirements.
        unsafe { lib.voicevox_user_dict_delete(dict) };
        unsafe { lib.voicevox_user_dict_delete(other_dict) };

        return Ok(());

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(c_api::VoicevoxResultCode_VOICEVOX_RESULT_OK, result_code);
        }

        fn assert_contains_uuid(text: &str, pattern: &Uuid) {
            assert!(text.contains(pattern.to_string().as_str()));
        }

        fn assert_not_contains_uuid(text: &str, pattern: &Uuid) {
            assert!(!text.contains(pattern.to_string().as_str()));
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

static SNAPSHOTS: LazyLock<Snapshots> = snapshots::section!(user_dict_manipulate);

#[derive(Deserialize)]
struct Snapshots {
    stderr: String,
}
