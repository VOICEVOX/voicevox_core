use std::os::raw::c_char;

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum VoicevoxResultCode {
    // C でのenum定義に合わせて大文字で定義している
    // 出力フォーマットを変更すればRustでよく使われているUpperCamelにできるが、実際に出力されるコードとの差異をできるだけ少なくするため
    VOICEVOX_RESULT_SUCCEED = 0,
    VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT = 1,
}

#[no_mangle]
pub extern "C" fn initialize(use_gpu: bool, cpu_num_threads: usize, load_all_models: bool) -> bool {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn load_model(speaker_id: i64) -> bool {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn is_model_loaded(speaker_id: i64) -> bool {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn finalize() {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn metas() -> *const c_char {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn last_error_message() -> *const c_char {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn supported_devices() -> *const c_char {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn yukarin_s_forward(
    length: i64,
    phoneme_list: *const i64,
    speaker_id: *const i64,
    output: *mut f32,
) -> VoicevoxResultCode {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn yukarin_sa_forward(
    length: i64,
    vowel_phoneme_list: *const i64,
    consonant_phoneme_list: *const i64,
    start_accent_list: *const i64,
    end_accent_list: *const i64,
    start_accent_phrase_list: *const i64,
    end_accent_phrase_list: *const i64,
    speaker_id: *const i64,
    output: *mut f32,
) -> VoicevoxResultCode {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn decode_forward(
    length: i64,
    phoneme_size: i64,
    f0: *const f32,
    phoneme: *const f32,
    speaker_id: *const i64,
    output: *mut f32,
) -> VoicevoxResultCode {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn voicevox_tts(
    text: *const c_char,
    speaker_id: i64,
    output_binary_size: *mut usize,
    output_wav: *const *mut u8,
) -> VoicevoxResultCode {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn voicevox_tts_from_kana(
    text: *const c_char,
    speaker_id: i64,
    output_binary_size: *mut usize,
    output_wav: *const *mut u8,
) -> VoicevoxResultCode {
    unimplemented!()
}

pub extern "C" fn voicevox_wav_free(wav: *mut u8) -> VoicevoxResultCode {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn voicevox_error_result_to_message(
    result_code: VoicevoxResultCode,
) -> *const c_char {
    unimplemented!()
}
