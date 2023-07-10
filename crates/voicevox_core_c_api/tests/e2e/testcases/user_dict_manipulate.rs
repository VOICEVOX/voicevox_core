// ユーザー辞書の操作をテストする。

use assert_cmd::assert::AssertResult;
use once_cell::sync::Lazy;
use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
};
use tempfile::{NamedTempFile, TempPath};
use voicevox_core::result_code::VoicevoxResultCode;

use libloading::Library;
use serde::{Deserialize, Serialize};

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
    symbols::{Symbols, VoicevoxUserDict, VoicevoxUserDictWordType},
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "user_dict_manipulate")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: &Library) -> anyhow::Result<()> {
        let Symbols {
            voicevox_default_user_dict_word,
            voicevox_dict_new,
            voicevox_dict_add_word,
            voicevox_dict_update_word,
            voicevox_dict_remove_word,
            voicevox_dict_get_words_json,
            voicevox_dict_merge,
            voicevox_dict_delete,
            ..
        } = Symbols::new(lib)?;

        let get_json = |dict: &*mut VoicevoxUserDict| -> &str {
            let mut json = std::ptr::null_mut();
            assert_ok(voicevox_dict_get_words_json((*dict) as *const _, &mut json));

            CStr::from_ptr(json).to_str().unwrap()
        };

        // テスト用の辞書ファイルを作成
        let dict_path = temp_dict_path();
        let dict = {
            let mut dict = MaybeUninit::uninit();
            assert_ok(voicevox_dict_new(
                CString::new(dict_path.to_str().unwrap())
                    .unwrap()
                    .into_raw(),
                dict.as_mut_ptr(),
            ));
            dict.assume_init()
        };

        // 単語の追加のテスト
        let mut word_uuid = std::ptr::null_mut();

        let word = {
            let mut word = voicevox_default_user_dict_word();
            word.surface = CString::new("hoge").unwrap().into_raw();
            word.pronunciation = CString::new("ホゲ").unwrap().into_raw();
            word.word_type = VoicevoxUserDictWordType::VOICEVOX_USER_DICT_WORD_TYPE_PROPER_NOUN;

            word
        };

        assert_ok(voicevox_dict_add_word(dict, &word, &mut word_uuid));

        let word_uuid = CStr::from_ptr(word_uuid).to_str().unwrap();

        let json = get_json(&dict);

        assert!(json.contains("ｈｏｇｅ"));
        assert!(json.contains("ホゲ"));
        assert!(json.contains(word_uuid));

        // 単語の変更のテスト
        let word = {
            let mut word = voicevox_default_user_dict_word();
            word.surface = CString::new("fuga").unwrap().into_raw();
            word.pronunciation = CString::new("フガ").unwrap().into_raw();
            word.word_type = VoicevoxUserDictWordType::VOICEVOX_USER_DICT_WORD_TYPE_COMMON_NOUN;

            word
        };

        assert_ok(voicevox_dict_update_word(
            dict,
            word_uuid.as_bytes().as_ptr() as *const i8,
            &word,
        ));

        let json = get_json(&dict);

        assert!(!json.contains("ｈｏｇｅ"));
        assert!(!json.contains("ホゲ"));
        assert!(json.contains("ｆｕｇａ"));
        assert!(json.contains("フガ"));
        assert!(json.contains(word_uuid));

        // 辞書のインポートのテスト。
        let other_dict_path = temp_dict_path();
        let other_dict = {
            let mut dict = MaybeUninit::uninit();
            assert_ok(voicevox_dict_new(
                CString::new(other_dict_path.as_os_str().to_str().unwrap())
                    .unwrap()
                    .into_raw(),
                dict.as_mut_ptr(),
            ));
            dict.assume_init()
        };

        let mut other_word_uuid = std::ptr::null_mut();

        let other_word = {
            let mut word = voicevox_default_user_dict_word();
            word.surface = CString::new("piyo").unwrap().into_raw();
            word.pronunciation = CString::new("ピヨ").unwrap().into_raw();

            word
        };

        assert_ok(voicevox_dict_add_word(
            other_dict,
            &other_word,
            &mut other_word_uuid as *mut *mut i8,
        ));

        let other_word_uuid = CStr::from_ptr(other_word_uuid).to_str().unwrap();

        assert_ok(voicevox_dict_merge(dict, other_dict));

        let json = get_json(&dict);
        assert!(json.contains("ｆｕｇａ"));
        assert!(json.contains("フガ"));
        assert!(json.contains(word_uuid));
        assert!(json.contains("ｐｉｙｏ"));
        assert!(json.contains("ピヨ"));
        assert!(json.contains(other_word_uuid));

        // 単語の削除のテスト
        assert_ok(voicevox_dict_remove_word(
            dict,
            word_uuid.as_bytes().as_ptr() as *const i8,
        ));

        let json = get_json(&dict);
        assert!(!json.contains(word_uuid));
        // 他の単語は残っている
        assert!(json.contains(other_word_uuid));

        voicevox_dict_delete(dict);
        voicevox_dict_delete(other_dict);

        return Ok(());

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(VoicevoxResultCode::VOICEVOX_RESULT_OK, result_code);
        }

        fn temp_dict_path() -> TempPath {
            NamedTempFile::new().unwrap().into_temp_path()
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
