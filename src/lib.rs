use std::os::raw::{c_char, c_void};

#[repr(C)]
pub struct VoicevoxInitializeOptions {
    use_gpu: bool,
    cpu_num_threads: usize,
    openjtalk_dict_path: *const c_char,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum VoicevoxResultCode {
    // C でのenum定義に合わせて大文字で定義している
    // 出力フォーマットを変更すればRustでよく使われているUpperCamelにできるが、実際に出力されるコードとの差異をできるだけ少なくするため
    VOICEVOX_RESULT_SUCCEED = 0,
    VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT = 1,
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn voicevox_default_initialize_options() -> VoicevoxInitializeOptions {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn voicevox_initialize(
    options: *const VoicevoxInitializeOptions,
) -> VoicevoxResultCode {
    unimplemented!()
}

type VoicevoxAsyncContext = *mut c_void;

#[repr(C)]
pub struct VoicevoxLoadModelAsyncResult {
    ctx: VoicevoxAsyncContext,
    result_code: VoicevoxResultCode,
}

#[no_mangle]
pub extern "C" fn voicevox_load_model_async(
    ctx: VoicevoxAsyncContext,
    speaker_id: i64,
    loaded_model_callback: extern "C" fn(*const VoicevoxLoadModelAsyncResult),
) -> VoicevoxResultCode {
    unimplemented!()
}
#[no_mangle]
pub extern "C" fn voicevox_finalize() -> VoicevoxResultCode {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn voicevox_metas() -> *const c_char {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn voicevox_supported_devices() -> *const c_char {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn voicevox_yukarin_s_forward(
    length: i64,
    phoneme_list: *const i64,
    speaker_id: *const i64,
    output: *mut f32,
) -> VoicevoxResultCode {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn voicevox_yukarin_sa_forward(
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
pub extern "C" fn voicevox_decode_forward(
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
