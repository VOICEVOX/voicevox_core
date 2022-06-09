use super::*;
use internal::Internal;
use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::{Mutex, MutexGuard};

static INTERNAL: Lazy<Mutex<Internal>> = Lazy::new(Internal::new_with_mutex);

fn lock_internal() -> MutexGuard<'static, Internal> {
    INTERNAL.lock().unwrap()
}

/*
 * Cの関数として公開するための型や関数を定義するこれらの実装はinternal.rsに定義してある同名関数にある
 * この関数ではinternal.rsにある同名関数の呼び出しと、その戻り値をCの形式に変換する処理のみとする
 * これはC文脈の処理と実装をわけるためと、内部実装の変更がAPIに影響を与えにくくするためである
 */

#[repr(i32)]
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum VoicevoxResultCode {
    // C でのenum定義に合わせて大文字で定義している
    // 出力フォーマットを変更すればRustでよく使われているUpperCamelにできるが、実際に出力されるコードとの差異をできるだけ少なくするため
    VOICEVOX_RESULT_SUCCEED = 0,
    VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT = 1,
    VOICEVOX_RESULT_FAILED_LOAD_MODEL = 2,
    VOICEVOX_RESULT_FAILED_GET_SUPPORTED_DEVICES = 3,
    VOICEVOX_RESULT_CANT_GPU_SUPPORT = 4,
    VOICEVOX_RESULT_FAILED_LOAD_METAS = 5,
    VOICEVOX_RESULT_UNINITIALIZED_STATUS = 6,
}

fn convert_result<T>(result: Result<T>) -> (Option<T>, VoicevoxResultCode) {
    match result {
        Ok(target) => (Some(target), VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED),
        Err(err) => {
            eprintln!("{}", err);
            dbg!(&err);
            match err {
                Error::NotLoadedOpenjtalkDict => (
                    None,
                    VoicevoxResultCode::VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT,
                ),
                Error::CantGpuSupport => {
                    (None, VoicevoxResultCode::VOICEVOX_RESULT_CANT_GPU_SUPPORT)
                }
                Error::LoadModel(_) => {
                    (None, VoicevoxResultCode::VOICEVOX_RESULT_FAILED_LOAD_MODEL)
                }
                Error::LoadMetas(_) => {
                    (None, VoicevoxResultCode::VOICEVOX_RESULT_FAILED_LOAD_METAS)
                }
                Error::GetSupportedDevices(_) => (
                    None,
                    VoicevoxResultCode::VOICEVOX_RESULT_FAILED_GET_SUPPORTED_DEVICES,
                ),
                Error::UninitializedStatus => (
                    None,
                    VoicevoxResultCode::VOICEVOX_RESULT_UNINITIALIZED_STATUS,
                ),
            }
        }
    }
}

// FIXME:各関数の戻り値をboolからVoicevoxResultCodeに変えてこのstatic変数を削除する
static ERROR_MESSAGE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

fn set_message(message: &str) {
    ERROR_MESSAGE
        .lock()
        .unwrap()
        .replace_range(.., &format!("{}\0", message));
}

#[no_mangle]
pub extern "C" fn initialize(use_gpu: bool, cpu_num_threads: c_int, load_all_models: bool) -> bool {
    let result = lock_internal().initialize(use_gpu, cpu_num_threads as usize, load_all_models);
    //TODO: VoicevoxResultCodeを返すようにする
    if let Some(err) = result.err() {
        set_message(&format!("{}", err));
        false
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn load_model(speaker_id: i64) -> bool {
    let result = lock_internal().load_model(speaker_id as usize);
    //TODO: VoicevoxResultCodeを返すようにする
    if let Some(err) = result.err() {
        set_message(&format!("{}", err));
        false
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn is_model_loaded(speaker_id: i64) -> bool {
    lock_internal().is_model_loaded(speaker_id as usize)
}

#[no_mangle]
pub extern "C" fn finalize() {
    lock_internal().finalize()
}

#[no_mangle]
pub extern "C" fn metas() -> *const c_char {
    lock_internal().metas().as_ptr()
}

#[no_mangle]
pub extern "C" fn last_error_message() -> *const c_char {
    ERROR_MESSAGE.lock().unwrap().as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn supported_devices() -> *const c_char {
    lock_internal().supported_devices().as_ptr()
}

#[no_mangle]
pub extern "C" fn yukarin_s_forward(
    length: i64,
    phoneme_list: *mut i64,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    let result =
        lock_internal().yukarin_s_forward(length, phoneme_list, &unsafe { *speaker_id }, output);
    //TODO: VoicevoxResultCodeを返すようにする
    if let Some(err) = result.err() {
        set_message(&format!("{}", err));
        false
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn yukarin_sa_forward(
    length: i64,
    vowel_phoneme_list: *mut i64,
    consonant_phoneme_list: *mut i64,
    start_accent_list: *mut i64,
    end_accent_list: *mut i64,
    start_accent_phrase_list: *mut i64,
    end_accent_phrase_list: *mut i64,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    let result = lock_internal().yukarin_sa_forward(
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
    //TODO: VoicevoxResultCodeを返すようにする
    if let Some(err) = result.err() {
        set_message(&format!("{}", err));
        false
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn decode_forward(
    length: i64,
    phoneme_size: i64,
    f0: *mut f32,
    phoneme: *mut f32,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    let result =
        lock_internal().decode_forward(length, phoneme_size, f0, phoneme, speaker_id, output);
    //TODO: VoicevoxResultCodeを返すようにする
    if let Some(err) = result.err() {
        set_message(&format!("{}", err));
        false
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn voicevox_load_openjtalk_dict(dict_path: *const c_char) -> VoicevoxResultCode {
    let (_, result_code) = convert_result(
        lock_internal().voicevox_load_openjtalk_dict(unsafe { CStr::from_ptr(dict_path) }),
    );
    result_code
}

#[no_mangle]
pub extern "C" fn voicevox_tts(
    text: *const c_char,
    speaker_id: i64,
    output_binary_size: *mut c_int,
    output_wav: *mut *mut u8,
) -> VoicevoxResultCode {
    let (_, result_code) = convert_result(lock_internal().voicevox_tts(
        unsafe { CStr::from_ptr(text) },
        speaker_id,
        output_binary_size,
        output_wav,
    ));
    result_code
}

#[no_mangle]
pub extern "C" fn voicevox_tts_from_kana(
    text: *const c_char,
    speaker_id: i64,
    output_binary_size: *mut c_int,
    output_wav: *mut *mut u8,
) -> VoicevoxResultCode {
    let (_, result_code) = convert_result(lock_internal().voicevox_tts_from_kana(
        unsafe { CStr::from_ptr(text) },
        speaker_id,
        output_binary_size,
        output_wav,
    ));
    result_code
}

#[no_mangle]
pub extern "C" fn voicevox_wav_free(wav: *mut u8) -> VoicevoxResultCode {
    let (_, result_code) = convert_result(lock_internal().voicevox_wav_free(wav));
    result_code
}

#[no_mangle]
pub extern "C" fn voicevox_error_result_to_message(
    result_code: VoicevoxResultCode,
) -> *const c_char {
    internal::voicevox_error_result_to_message(result_code).as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use pretty_assertions::assert_eq;

    #[rstest]
    #[case(Ok(()), VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED)]
    #[case(
        Err(Error::NotLoadedOpenjtalkDict),
        VoicevoxResultCode::VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT
    )]
    #[case(
        Err(Error::LoadModel(anyhow!("some load model error"))),
        VoicevoxResultCode::VOICEVOX_RESULT_FAILED_LOAD_MODEL
    )]
    #[case(
        Err(Error::GetSupportedDevices(anyhow!("some get supported devices error"))),
        VoicevoxResultCode::VOICEVOX_RESULT_FAILED_GET_SUPPORTED_DEVICES
    )]
    fn convert_result_works(#[case] result: Result<()>, #[case] expected: VoicevoxResultCode) {
        let (_, actual) = convert_result(result);
        assert_eq!(expected, actual);
    }
}
