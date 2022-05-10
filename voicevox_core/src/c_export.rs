use super::*;
use std::ffi::CStr;
use std::os::raw::c_char;

/*
 * Cの関数として公開するための型や関数を定義するこれらの実装はinternal.rsに定義してある同名関数にある
 * この関数ではinternal.rsにある同名関数の呼び出しと、その戻り値をCの形式に変換する処理のみとする
 * これはC文脈の処理と実装をわけるためと、内部実装の変更がAPIに影響を与えにくくするためである
 */

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum VoicevoxResultCode {
    // C でのenum定義に合わせて大文字で定義している
    // 出力フォーマットを変更すればRustでよく使われているUpperCamelにできるが、実際に出力されるコードとの差異をできるだけ少なくするため
    VOICEVOX_RESULT_SUCCEED = 0,
    VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT = 1,
}

impl<T> From<Result<T>> for VoicevoxResultCode {
    fn from(result: Result<T>) -> Self {
        if let Some(err) = result.err() {
            eprintln!("{}", err);
            dbg!(&err);
            if let Ok(err) = err.downcast::<Error>() {
                match err {
                    Error::NotLoadedOpenjtalkDict => {
                        VoicevoxResultCode::VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT
                    }
                }
            } else {
                panic!()
            }
        } else {
            VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED
        }
    }
}

// FIXME:static変数はunsafeなので各関数の戻り値をboolからVoicevoxResultCodeに変えてこのstatic変数を削除する
static mut ERROR_MESSAGE: String = String::new();

#[no_mangle]
pub extern "C" fn initialize(use_gpu: bool, cpu_num_threads: usize, load_all_models: bool) -> bool {
    let result = internal::initialize(use_gpu, cpu_num_threads, load_all_models);
    if let Some(err) = result.err() {
        unsafe {
            ERROR_MESSAGE = format!("{}\0", err);
        }
        false
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn load_model(speaker_id: i64) -> bool {
    let result = internal::load_model(speaker_id);
    if let Some(err) = result.err() {
        unsafe {
            ERROR_MESSAGE = format!("{}\0", err);
        }
        false
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn is_model_loaded(speaker_id: i64) -> bool {
    internal::is_model_loaded(speaker_id)
}

#[no_mangle]
pub extern "C" fn finalize() {
    internal::finalize()
}

#[no_mangle]
pub extern "C" fn metas() -> *const c_char {
    internal::metas().as_ptr()
}

#[no_mangle]
pub extern "C" fn last_error_message() -> *const c_char {
    unsafe { ERROR_MESSAGE.as_ptr() as *const c_char }
}

#[no_mangle]
pub extern "C" fn supported_devices() -> *const c_char {
    internal::supported_devices().as_ptr()
}

#[no_mangle]
pub extern "C" fn yukarin_s_forward(
    length: i64,
    phoneme_list: *const i64,
    speaker_id: *const i64,
    output: *mut f32,
) -> bool {
    let result = internal::yukarin_s_forward(length, phoneme_list, speaker_id, output);
    if let Some(err) = result.err() {
        unsafe {
            ERROR_MESSAGE = format!("{}\0", err);
        }
        false
    } else {
        true
    }
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
) -> bool {
    let result = internal::yukarin_sa_forward(
        length,
        vowel_phoneme_list,
        consonant_phoneme_list,
        start_accent_list,
        end_accent_list,
        start_accent_phrase_list,
        end_accent_phrase_list,
        speaker_id,
        output,
    );
    if let Some(err) = result.err() {
        unsafe {
            ERROR_MESSAGE = format!("{}\0", err);
        }
        false
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn decode_forward(
    length: i64,
    phoneme_size: i64,
    f0: *const f32,
    phoneme: *const f32,
    speaker_id: *const i64,
    output: *mut f32,
) -> bool {
    let result = internal::decode_forward(length, phoneme_size, f0, phoneme, speaker_id, output);
    if let Some(err) = result.err() {
        unsafe {
            ERROR_MESSAGE = format!("{}\0", err);
        }
        false
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn voicevox_load_openjtalk_dict(dict_path: *const c_char) -> VoicevoxResultCode {
    let dict_path = unsafe { CStr::from_ptr(dict_path).to_str().unwrap() };
    internal::voicevox_load_openjtalk_dict(dict_path).into()
}

#[no_mangle]
pub extern "C" fn voicevox_tts(
    text: *const c_char,
    speaker_id: i64,
    output_binary_size: *mut usize,
    output_wav: *const *mut u8,
) -> VoicevoxResultCode {
    internal::voicevox_tts(
        unsafe { CStr::from_ptr(text) },
        speaker_id,
        output_binary_size,
        output_wav,
    )
    .into()
}

#[no_mangle]
pub extern "C" fn voicevox_tts_from_kana(
    text: *const c_char,
    speaker_id: i64,
    output_binary_size: *mut usize,
    output_wav: *const *mut u8,
) -> VoicevoxResultCode {
    internal::voicevox_tts_from_kana(
        unsafe { CStr::from_ptr(text) },
        speaker_id,
        output_binary_size,
        output_wav,
    )
    .into()
}

pub extern "C" fn voicevox_wav_free(wav: *mut u8) -> VoicevoxResultCode {
    internal::voicevox_wav_free(wav).into()
}

#[no_mangle]
pub extern "C" fn voicevox_error_result_to_message(
    result_code: VoicevoxResultCode,
) -> *const c_char {
    internal::voicevox_error_result_to_message(result_code)
}
