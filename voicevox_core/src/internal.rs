use super::*;
use c_export::VoicevoxResultCode;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::path::Path;

pub fn initialize(use_gpu: bool, cpu_num_threads: usize, load_all_models: bool) -> Result<()> {
    unimplemented!()
}

pub fn load_model(speaker_id: i64) -> Result<()> {
    unimplemented!()
}

pub fn is_model_loaded(speaker_id: i64) -> bool {
    unimplemented!()
}

pub fn finalize() {
    unimplemented!()
}

pub fn metas() -> &'static CStr {
    unimplemented!()
}

pub fn supported_devices() -> &'static CStr {
    unimplemented!()
}

pub fn yukarin_s_forward(
    length: i64,
    phoneme_list: *const i64,
    speaker_id: &i64,
    output: *mut f32,
) -> Result<()> {
    unimplemented!()
}

#[allow(clippy::too_many_arguments)]
pub fn yukarin_sa_forward(
    length: i64,
    vowel_phoneme_list: *const i64,
    consonant_phoneme_list: *const i64,
    start_accent_list: *const i64,
    end_accent_list: *const i64,
    start_accent_phrase_list: *const i64,
    end_accent_phrase_list: *const i64,
    speaker_id: *const i64,
    output: *mut f32,
) -> Result<()> {
    unimplemented!()
}

pub fn decode_forward(
    length: i64,
    phoneme_size: i64,
    f0: *const f32,
    phoneme: *const f32,
    speaker_id: *const i64,
    output: *mut f32,
) -> Result<()> {
    unimplemented!()
}

pub fn voicevox_load_openjtalk_dict(dict_path: &CStr) -> Result<()> {
    unimplemented!()
}
pub fn voicevox_tts(
    text: &CStr,
    speaker_id: i64,
    output_binary_size: *mut c_int,
    output_wav: *const *mut u8,
) -> Result<()> {
    unimplemented!()
}

pub fn voicevox_tts_from_kana(
    text: &CStr,
    speaker_id: i64,
    output_binary_size: *mut usize,
    output_wav: *const *mut u8,
) -> Result<()> {
    unimplemented!()
}

pub fn voicevox_wav_free(wav: *mut u8) -> Result<()> {
    unimplemented!()
}

pub const fn voicevox_error_result_to_message(result_code: VoicevoxResultCode) -> &'static str {
    // C APIのため、messageには必ず末尾にNULL文字を追加する
    match result_code {
        VoicevoxResultCode::VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT => {
            "voicevox_load_openjtalk_dict() を初めに呼んでください\0"
        }

        VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED => "エラーが発生しませんでした\0",
    }
}
