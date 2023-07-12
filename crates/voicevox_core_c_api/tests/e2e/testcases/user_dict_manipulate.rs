// ユーザー辞書の操作をテストする。

use assert_cmd::assert::AssertResult;
use once_cell::sync::Lazy;
use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
};
use tempfile::NamedTempFile;
use voicevox_core::result_code::VoicevoxResultCode;

use libloading::Library;
use serde::{Deserialize, Serialize};

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
    symbols::{Symbols, VoicevoxUserDict, VoicevoxUserDictWord, VoicevoxUserDictWordType},
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "user_dict_manipulate")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: &Library) -> anyhow::Result<()> {
        let Symbols {
            voicevox_default_user_dict_word,
            voicevox_user_dict_new,
            voicevox_user_dict_add_word,
            voicevox_user_dict_update_word,
            voicevox_user_dict_remove_word,
            voicevox_user_dict_get_json,
            voicevox_user_dict_import,
            voicevox_user_dict_load,
            voicevox_user_dict_save,
            voicevox_user_dict_delete,
            voicevox_user_dict_uuid_free,
            voicevox_json_free,
            ..
        } = Symbols::new(lib)?;

        let get_json = |dict: &*mut VoicevoxUserDict| -> String {
            let mut json = MaybeUninit::uninit();
            assert_ok(voicevox_user_dict_get_json(
                (*dict) as *const _,
                json.as_mut_ptr(),
            ));

            let ret = CStr::from_ptr(json.assume_init())
                .to_str()
                .unwrap()
                .to_string();

            voicevox_json_free(json.assume_init());

            serde_json::from_str::<serde_json::Value>(&ret).expect("invalid json");

            ret
        };

        let add_word = |dict: &*mut VoicevoxUserDict, word: &VoicevoxUserDictWord| -> CString {
            let mut word_uuid = MaybeUninit::uninit();

            assert_ok(voicevox_user_dict_add_word(
                (*dict) as *const _,
                word as *const _,
                word_uuid.as_mut_ptr(),
            ));

            let ret = CStr::from_ptr(word_uuid.assume_init()).to_owned();

            voicevox_user_dict_uuid_free(word_uuid.assume_init());

            ret
        };

        // テスト用の辞書ファイルを作成
        let dict = {
            let mut dict = MaybeUninit::uninit();
            assert_ok(voicevox_user_dict_new(dict.as_mut_ptr()));
            dict.assume_init()
        };

        // 単語の追加のテスト
        let word = {
            let mut word = voicevox_default_user_dict_word();
            word.surface = CString::new("hoge").unwrap().into_raw();
            word.pronunciation = CString::new("ホゲ").unwrap().into_raw();
            word.word_type = VoicevoxUserDictWordType::VOICEVOX_USER_DICT_WORD_TYPE_PROPER_NOUN;

            word
        };

        let word_uuid = add_word(&dict, &word);

        let json = get_json(&dict);

        assert!(json.contains("ｈｏｇｅ"));
        assert!(json.contains("ホゲ"));
        assert_contains_cstring(&json, &word_uuid);

        // 単語の変更のテスト
        let word = {
            let mut word = voicevox_default_user_dict_word();
            word.surface = CString::new("fuga").unwrap().into_raw();
            word.pronunciation = CString::new("フガ").unwrap().into_raw();
            word.word_type = VoicevoxUserDictWordType::VOICEVOX_USER_DICT_WORD_TYPE_COMMON_NOUN;

            word
        };

        assert_ok(voicevox_user_dict_update_word(
            dict,
            word_uuid.as_ptr(),
            &word,
        ));

        let json = get_json(&dict);

        assert!(!json.contains("ｈｏｇｅ"));
        assert!(!json.contains("ホゲ"));
        assert!(json.contains("ｆｕｇａ"));
        assert!(json.contains("フガ"));
        assert_contains_cstring(&json, &word_uuid);

        // 辞書のインポートのテスト。
        let other_dict = {
            let mut dict = MaybeUninit::uninit();
            assert_ok(voicevox_user_dict_new(dict.as_mut_ptr()));
            dict.assume_init()
        };

        let other_word = {
            let mut word = voicevox_default_user_dict_word();
            word.surface = CString::new("piyo").unwrap().into_raw();
            word.pronunciation = CString::new("ピヨ").unwrap().into_raw();

            word
        };

        let other_word_uuid = add_word(&other_dict, &other_word);

        assert_ok(voicevox_user_dict_import(dict, other_dict));

        let json = get_json(&dict);
        assert!(json.contains("ｆｕｇａ"));
        assert!(json.contains("フガ"));
        assert_contains_cstring(&json, &word_uuid);
        assert!(json.contains("ｐｉｙｏ"));
        assert!(json.contains("ピヨ"));
        assert_contains_cstring(&json, &other_word_uuid);

        // 単語の削除のテスト
        assert_ok(voicevox_user_dict_remove_word(dict, word_uuid.as_ptr()));

        let json = get_json(&dict);
        assert_not_contains_cstring(&json, &word_uuid);
        // 他の単語は残っている
        assert_contains_cstring(&json, &other_word_uuid);

        // 辞書のセーブ・ロードのテスト
        let temp_path = NamedTempFile::new().unwrap().into_temp_path();
        let temp_path = CString::new(temp_path.to_str().unwrap()).unwrap();
        let word = {
            let mut word = voicevox_default_user_dict_word();
            word.surface = CString::new("foo").unwrap().into_raw();
            word.pronunciation = CString::new("フー").unwrap().into_raw();
            word.word_type = VoicevoxUserDictWordType::VOICEVOX_USER_DICT_WORD_TYPE_PROPER_NOUN;

            word
        };
        let word_uuid = add_word(&dict, &word);

        assert_ok(voicevox_user_dict_save(
            dict,
            temp_path.as_ptr() as *const i8,
        ));
        assert_ok(voicevox_user_dict_load(
            other_dict,
            temp_path.as_ptr() as *const i8,
        ));

        let json = get_json(&other_dict);
        assert_contains_cstring(&json, &word_uuid);
        assert_contains_cstring(&json, &other_word_uuid);

        voicevox_user_dict_delete(dict);
        voicevox_user_dict_delete(other_dict);

        return Ok(());

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(VoicevoxResultCode::VOICEVOX_RESULT_OK, result_code);
        }

        fn assert_contains_cstring(text: &str, pattern: &CString) {
            assert!(text.contains(pattern.to_str().unwrap()));
        }

        fn assert_not_contains_cstring(text: &str, pattern: &CString) {
            assert!(!text.contains(pattern.to_str().unwrap()));
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

static SNAPSHOTS: Lazy<Snapshots> = snapshots::section!(user_dict_manipulate);

#[derive(Deserialize)]
struct Snapshots {
    stderr: String,
}
