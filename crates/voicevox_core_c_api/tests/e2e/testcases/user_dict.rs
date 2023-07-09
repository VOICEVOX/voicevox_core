// ユーザー辞書の登録によってAudioQueryが変化することを確認するテスト

use crate::symbols::VoicevoxInitializeOptions;
use assert_cmd::assert::AssertResult;
use once_cell::sync::Lazy;
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use tempfile::NamedTempFile;
use test_util::OPEN_JTALK_DIC_DIR;
use voicevox_core::result_code::VoicevoxResultCode;

use libloading::Library;
use serde::{Deserialize, Serialize};

use crate::{
    assert_cdylib::{self, case, Utf8Output},
    snapshots,
    symbols::{Symbols, VoicevoxAccelerationMode, VoicevoxUserDictWordType},
};

macro_rules! cstr {
    ($s:literal $(,)?) => {
        CStr::from_bytes_with_nul(concat!($s, '\0').as_ref()).unwrap()
    };
}

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

            voicevox_default_initialize_options,
            voicevox_default_audio_query_options,
            voicevox_open_jtalk_rc_new,
            voicevox_open_jtalk_rc_load_user_dict,
            voicevox_open_jtalk_rc_delete,
            voicevox_voice_model_new_from_path,
            voicevox_voice_model_delete,
            voicevox_synthesizer_new_with_initialize,
            voicevox_synthesizer_delete,
            voicevox_synthesizer_load_voice_model,
            voicevox_synthesizer_audio_query,
            ..
        } = Symbols::new(lib)?;

        let mut dict = std::ptr::null_mut();

        let temp_dict_path = NamedTempFile::new()?.into_temp_path();
        let temp_dict_path_cstr =
            CStr::from_bytes_with_nul_unchecked(temp_dict_path.to_str().unwrap().as_bytes());
        assert_ok(voicevox_dict_new(temp_dict_path_cstr.as_ptr(), &mut dict));

        let mut word = voicevox_default_user_dict_word();
        let mut word_uuid = std::ptr::null_mut();

        word.surface = CString::new("this_word_should_not_exist_in_default_dictionary")
            .unwrap()
            .into_raw();
        word.pronunciation = CString::new("アイウエオ").unwrap().into_raw();
        word.word_type = VoicevoxUserDictWordType::VOICEVOX_USER_DICT_WORD_TYPE_PROPER_NOUN;
        word.priority = 10;

        assert_ok(voicevox_dict_add_word(dict, &word, &mut word_uuid));

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

        assert_ok(voicevox_open_jtalk_rc_load_user_dict(openjtalk, dict));

        let synthesizer = {
            let mut synthesizer = MaybeUninit::uninit();
            assert_ok(voicevox_synthesizer_new_with_initialize(
                openjtalk,
                VoicevoxInitializeOptions {
                    acceleration_mode: VoicevoxAccelerationMode::VOICEVOX_ACCELERATION_MODE_CPU,
                    ..**voicevox_default_initialize_options
                },
                synthesizer.as_mut_ptr(),
            ));
            synthesizer.assume_init()
        };

        assert_ok(voicevox_synthesizer_load_voice_model(synthesizer, model));

        let mut audio_query_without_dict = std::ptr::null_mut();
        assert_ok(voicevox_synthesizer_audio_query(
            synthesizer,
            cstr!("this_word_should_not_exist_in_default_dictionary").as_ptr(),
            STYLE_ID,
            **voicevox_default_audio_query_options,
            &mut audio_query_without_dict,
        ));
        let audio_query_without_dict = serde_json::from_str::<serde_json::Value>(
            &CString::from_raw(audio_query_without_dict).into_string()?,
        )?;

        let mut audio_query_with_dict = std::ptr::null_mut();
        assert_ok(voicevox_synthesizer_audio_query(
            synthesizer,
            cstr!("this_word_should_not_exist_in_default_dictionary").as_ptr(),
            STYLE_ID,
            **voicevox_default_audio_query_options,
            &mut audio_query_with_dict,
        ));

        let audio_query_with_dict = serde_json::from_str::<serde_json::Value>(
            &CString::from_raw(audio_query_with_dict).into_string()?,
        )?;

        assert_ne!(
            audio_query_without_dict.get("kana"),
            audio_query_with_dict.get("kana")
        );

        voicevox_voice_model_delete(model);
        voicevox_open_jtalk_rc_delete(openjtalk);
        voicevox_synthesizer_delete(synthesizer);

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

static SNAPSHOTS: Lazy<Snapshots> = snapshots::section!(global_info);

#[derive(Deserialize)]
struct Snapshots {
    stderr: String,
}
