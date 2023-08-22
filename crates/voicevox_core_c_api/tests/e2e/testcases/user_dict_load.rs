// ユーザー辞書の登録によって読みが変化することを確認するテスト。
// 辞書ロード前後でAudioQueryのkanaが変化するかどうかで確認する。

use crate::symbols::{VoicevoxInitializeOptions, VoicevoxResultCode};
use assert_cmd::assert::AssertResult;
use once_cell::sync::Lazy;
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use test_util::OPEN_JTALK_DIC_DIR;

use cstr::cstr;
use libloading::Library;
use serde::{Deserialize, Serialize};

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
    symbols::{Symbols, VoicevoxAccelerationMode, VoicevoxUserDictWordType},
};

case!(TestCase);

#[derive(Serialize, Deserialize)]
struct TestCase;

#[typetag::serde(name = "user_dict_load")]
impl assert_cdylib::TestCase for TestCase {
    unsafe fn exec(&self, lib: &Library) -> anyhow::Result<()> {
        let Symbols {
            voicevox_user_dict_word_make,
            voicevox_user_dict_new,
            voicevox_user_dict_add_word,
            voicevox_user_dict_delete,
            voicevox_make_default_initialize_options,
            voicevox_make_default_audio_query_options,
            voicevox_open_jtalk_rc_new,
            voicevox_open_jtalk_rc_use_user_dict,
            voicevox_open_jtalk_rc_delete,
            voicevox_voice_model_new_from_path,
            voicevox_voice_model_delete,
            voicevox_synthesizer_new_with_initialize,
            voicevox_synthesizer_delete,
            voicevox_synthesizer_load_voice_model,
            voicevox_synthesizer_create_audio_query,
            ..
        } = Symbols::new(lib)?;

        let dict = voicevox_user_dict_new();

        let mut word_uuid = [0u8; 16];

        let word = {
            let mut word = voicevox_user_dict_word_make(
                cstr!("this_word_should_not_exist_in_default_dictionary").as_ptr(),
                cstr!("アイウエオ").as_ptr(),
            );
            word.word_type = VoicevoxUserDictWordType::VOICEVOX_USER_DICT_WORD_TYPE_PROPER_NOUN;
            word.priority = 10;

            word
        };

        assert_ok(voicevox_user_dict_add_word(dict, &word, &mut word_uuid));

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

        let mut audio_query_without_dict = std::ptr::null_mut();
        assert_ok(voicevox_synthesizer_create_audio_query(
            synthesizer,
            cstr!("this_word_should_not_exist_in_default_dictionary").as_ptr(),
            STYLE_ID,
            voicevox_make_default_audio_query_options(),
            &mut audio_query_without_dict,
        ));
        let audio_query_without_dict = serde_json::from_str::<serde_json::Value>(
            CStr::from_ptr(audio_query_without_dict).to_str()?,
        )?;

        assert_ok(voicevox_open_jtalk_rc_use_user_dict(openjtalk, dict));

        let mut audio_query_with_dict = std::ptr::null_mut();
        assert_ok(voicevox_synthesizer_create_audio_query(
            synthesizer,
            cstr!("this_word_should_not_exist_in_default_dictionary").as_ptr(),
            STYLE_ID,
            voicevox_make_default_audio_query_options(),
            &mut audio_query_with_dict,
        ));

        let audio_query_with_dict = serde_json::from_str::<serde_json::Value>(
            CStr::from_ptr(audio_query_with_dict).to_str()?,
        )?;

        assert_ne!(
            audio_query_without_dict.get("kana"),
            audio_query_with_dict.get("kana")
        );

        voicevox_voice_model_delete(model);
        voicevox_open_jtalk_rc_delete(openjtalk);
        voicevox_synthesizer_delete(synthesizer);
        voicevox_user_dict_delete(dict);

        return Ok(());

        fn assert_ok(result_code: VoicevoxResultCode) {
            std::assert_eq!(VoicevoxResultCode::VOICEVOX_RESULT_OK, result_code);
        }
        const STYLE_ID: u32 = 0;
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

static SNAPSHOTS: Lazy<Snapshots> = snapshots::section!(user_dict);

#[derive(Deserialize)]
struct Snapshots {
    #[serde(deserialize_with = "snapshots::deserialize_platform_specific_snapshot")]
    stderr: String,
}
