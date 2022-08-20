// TODO: ドキュメントを作成する段階になったらこのallowを外し、各pointerを使用している関数にunsafeとSafety documentを追加する
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use libc::c_void;
use once_cell::sync::Lazy;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::{Mutex, MutexGuard};
use voicevox_core::AudioQueryModel;
use voicevox_core::VoicevoxCore;
use voicevox_core::{Error, Result};

#[cfg(test)]
use rstest::*;

type Internal = VoicevoxCore;

static INTERNAL: Lazy<Mutex<Internal>> = Lazy::new(Internal::new_with_mutex);

fn lock_internal() -> MutexGuard<'static, Internal> {
    INTERNAL.lock().unwrap()
}

/*
 * Cの関数として公開するための型や関数を定義するこれらの実装はinternal.rsに定義してある同名関数にある
 * この関数ではinternal.rsにある同名関数の呼び出しと、その戻り値をCの形式に変換する処理のみとする
 * これはC文脈の処理と実装をわけるためと、内部実装の変更がAPIに影響を与えにくくするためである
 */

pub use voicevox_core::result_code::VoicevoxResultCode;

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
                Error::InvalidSpeakerId { .. } => {
                    (None, VoicevoxResultCode::VOICEVOX_RESULT_INVALID_SPEAKER_ID)
                }
                Error::InvalidModelIndex { .. } => (
                    None,
                    VoicevoxResultCode::VOICEVOX_RESULT_INVALID_MODEL_INDEX,
                ),
                Error::InferenceFailed => {
                    (None, VoicevoxResultCode::VOICEVOX_RESULT_INFERENCE_FAILED)
                }
                Error::FailedExtractFullContextLabel(_) => (
                    None,
                    VoicevoxResultCode::VOICEVOX_RESULT_FAILED_EXTRACT_FULL_CONTEXT_LABEL,
                ),
                Error::FailedParseKana(_) => {
                    (None, VoicevoxResultCode::VOICEVOX_RESULT_FAILED_PARSE_KANA)
                }
            }
        }
    }
}

#[repr(C)]
pub struct VoicevoxInitializeOptions {
    use_cuda: bool,
    cpu_num_threads: u32,
    load_all_models: bool,
    open_jtalk_dict_dir: *const c_char,
}

#[no_mangle]
pub extern "C" fn voicevox_default_initialize_options() -> VoicevoxInitializeOptions {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn voicevox_initialize(options: VoicevoxInitializeOptions) -> VoicevoxResultCode {
    unimplemented!();
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
pub extern "C" fn voicevox_load_model(speaker_id: i64) -> VoicevoxResultCode {
    unimplemented!();
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
pub extern "C" fn voicevox_is_model_loaded(speaker_id: i64) -> VoicevoxResultCode {
    unimplemented!();
    lock_internal().is_model_loaded(speaker_id as usize)
}

#[no_mangle]
pub extern "C" fn voicevox_finalize() {
    lock_internal().finalize()
}

#[no_mangle]
pub extern "C" fn voicevox_get_metas_json() -> *const c_char {
    lock_internal().metas().as_ptr()
}

#[no_mangle]
pub extern "C" fn voicevox_get_supported_devices_json() -> *const c_char {
    lock_internal().supported_devices().as_ptr()
}

#[no_mangle]
pub extern "C" fn voicevox_yukarin_s_forward(
    length: i64,
    phoneme_list: *mut i64,
    speaker_id: *mut i64,
    output: *mut f32,
) -> VoicevoxResultCode {
    unimplemented!();
    let result = lock_internal().yukarin_s_forward(
        unsafe { std::slice::from_raw_parts_mut(phoneme_list, length as usize) },
        unsafe { *speaker_id as usize },
    );
    //TODO: VoicevoxResultCodeを返すようにする
    match result {
        Ok(output_vec) => {
            let output_slice = unsafe { std::slice::from_raw_parts_mut(output, length as usize) };
            output_slice.clone_from_slice(&output_vec);
            true
        }
        Err(err) => {
            set_message(&format!("{}", err));
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn voicevox_yukarin_sa_forward(
    length: i64,
    vowel_phoneme_list: *mut i64,
    consonant_phoneme_list: *mut i64,
    start_accent_list: *mut i64,
    end_accent_list: *mut i64,
    start_accent_phrase_list: *mut i64,
    end_accent_phrase_list: *mut i64,
    speaker_id: *mut i64,
    output: *mut f32,
) -> VoicevoxResultCode {
    unimplemented!();
    let result = lock_internal().yukarin_sa_forward(
        length,
        unsafe { std::slice::from_raw_parts(vowel_phoneme_list, length as usize) },
        unsafe { std::slice::from_raw_parts(consonant_phoneme_list, length as usize) },
        unsafe { std::slice::from_raw_parts(start_accent_list, length as usize) },
        unsafe { std::slice::from_raw_parts(end_accent_list, length as usize) },
        unsafe { std::slice::from_raw_parts(start_accent_phrase_list, length as usize) },
        unsafe { std::slice::from_raw_parts(end_accent_phrase_list, length as usize) },
        unsafe { *speaker_id as usize },
    );
    //TODO: VoicevoxResultCodeを返すようにする
    match result {
        Ok(output_vec) => {
            let output_slice = unsafe { std::slice::from_raw_parts_mut(output, length as usize) };
            output_slice.clone_from_slice(&output_vec);
            true
        }
        Err(err) => {
            set_message(&format!("{}", err));
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn voicevox_decode_forward(
    length: i64,
    phoneme_size: i64,
    f0: *mut f32,
    phoneme: *mut f32,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    let length = length as usize;
    let phoneme_size = phoneme_size as usize;
    let result = lock_internal().decode_forward(
        length,
        phoneme_size,
        unsafe { std::slice::from_raw_parts(f0, length) },
        unsafe { std::slice::from_raw_parts(phoneme, phoneme_size * length) },
        unsafe { *speaker_id as usize },
    );
    //TODO: VoicevoxResultCodeを返すようにする
    match result {
        Ok(output_vec) => {
            let output_slice =
                unsafe { std::slice::from_raw_parts_mut(output, (length as usize) * 256) };
            output_slice.clone_from_slice(&output_vec);
            true
        }
        Err(err) => {
            set_message(&format!("{}", err));
            false
        }
    }
}

#[repr(C)]
pub struct VoicevoxAudioQueryOptions {
    kana: bool,
}

#[no_mangle]
pub extern "C" fn voicevox_default_audio_query_options() -> VoicevoxAudioQueryOptions {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn voicevox_audio_query(
    text: *const c_char,
    speaker_id: i64,
    options: VoicevoxAudioQueryOptions,
    output_audio_query_json: *mut *mut c_char,
) -> VoicevoxResultCode {
    unimplemented!();
    let text = unsafe { CStr::from_ptr(text) };

    let audio_query = &match create_audio_query(text, speaker_id, Internal::voicevox_audio_query) {
        Ok(audio_query) => audio_query,
        Err(result_code) => return result_code,
    };

    unsafe {
        write_json_to_ptr(output_audio_query_json, audio_query);
    }
    VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED
}

fn create_audio_query(
    japanese_or_kana: &CStr,
    speaker_id: i64,
    method: fn(&mut Internal, &str, usize) -> Result<AudioQueryModel>,
) -> std::result::Result<CString, VoicevoxResultCode> {
    let japanese_or_kana = ensure_utf8(japanese_or_kana)?;
    let speaker_id = speaker_id as usize;

    let (audio_query, result_code) =
        convert_result(method(&mut lock_internal(), japanese_or_kana, speaker_id));
    let audio_query = audio_query.ok_or(result_code)?;
    Ok(CString::new(audio_query_model_to_json(&audio_query)).expect("should not contain '\\0'"))
}

fn audio_query_model_to_json(audio_query_model: &AudioQueryModel) -> String {
    serde_json::to_string(audio_query_model).expect("should be always valid")
}

unsafe fn write_json_to_ptr(output_ptr: *mut *mut c_char, json: &CStr) {
    let n = json.to_bytes_with_nul().len();
    let json_heap = libc::malloc(n);
    libc::memcpy(json_heap, json.as_ptr() as *const c_void, n);
    output_ptr.write(json_heap as *mut c_char);
}

unsafe fn write_wav_to_ptr(output_wav_ptr: *mut *mut u8, output_size_ptr: *mut c_int, data: &[u8]) {
    output_size_ptr.write(data.len() as c_int);
    let wav_heap = libc::malloc(data.len());
    libc::memcpy(wav_heap, data.as_ptr() as *const c_void, data.len());
    output_wav_ptr.write(wav_heap as *mut u8);
}

#[no_mangle]
pub extern "C" fn voicevox_synthesis(
    audio_query_json: *const c_char,
    speaker_id: i64,
    output_binary_size: *mut c_int,
    output_wav: *mut *mut u8,
) -> VoicevoxResultCode {
    let audio_query_json = unsafe { CStr::from_ptr(audio_query_json) };

    let audio_query_json = match ensure_utf8(audio_query_json) {
        Ok(audio_query_json) => audio_query_json,
        Err(result_code) => return result_code,
    };
    let audio_query = &if let Ok(audio_query) = serde_json::from_str(audio_query_json) {
        audio_query
    } else {
        return VoicevoxResultCode::VOICEVOX_RESULT_INVALID_AUDIO_QUERY;
    };

    let speaker_id = speaker_id as usize;

    let (wav, result_code) =
        convert_result(lock_internal().voicevox_synthesis(audio_query, speaker_id));
    let wav = &if let Some(wav) = wav {
        wav
    } else {
        return result_code;
    };

    unsafe {
        write_wav_to_ptr(output_wav, output_binary_size, wav);
    }
    VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED
}

fn ensure_utf8(s: &CStr) -> std::result::Result<&str, VoicevoxResultCode> {
    s.to_str()
        .map_err(|_| VoicevoxResultCode::VOICEVOX_RESULT_INVALID_UTF8_INPUT)
}

#[repr(C)]
pub struct VoicevoxTtsOptions {
    kana: bool,
}

#[no_mangle]
pub fn voicevox_default_tts_options() -> VoicevoxTtsOptions {
    unimplemented!()
}

#[no_mangle]
pub extern "C" fn voicevox_tts(
    text: *const c_char,
    speaker_id: i64,
    options: VoicevoxTtsOptions,
    output_binary_size: *mut c_int,
    output_wav: *mut *mut u8,
) -> VoicevoxResultCode {
    let (output_opt, result_code) = {
        if let Ok(text) = unsafe { CStr::from_ptr(text) }.to_str() {
            convert_result(lock_internal().voicevox_tts(text, speaker_id as usize))
        } else {
            (None, VoicevoxResultCode::VOICEVOX_RESULT_INVALID_UTF8_INPUT)
        }
    };
    if let Some(output) = output_opt {
        unsafe {
            write_wav_to_ptr(output_wav, output_binary_size, output.as_slice());
        }
    }
    result_code
}

#[no_mangle]
pub extern "C" fn voicevox_audio_query_json_free(json: *mut c_char) {
    unsafe {
        libc::free(json as *mut c_void);
    }
}

#[no_mangle]
pub extern "C" fn voicevox_wav_free(wav: *mut u8) {
    unsafe {
        libc::free(wav as *mut c_void);
    }
}

#[no_mangle]
pub extern "C" fn voicevox_error_result_to_message(
    result_code: VoicevoxResultCode,
) -> *const c_char {
    voicevox_core::voicevox_error_result_to_message(result_code).as_ptr() as *const c_char
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
        Err(Error::LoadModel(voicevox_core::SourceError::new(anyhow!("some load model error")))),
        VoicevoxResultCode::VOICEVOX_RESULT_FAILED_LOAD_MODEL
    )]
    #[case(
        Err(Error::GetSupportedDevices(voicevox_core::SourceError::new(anyhow!("some get supported devices error")))),
        VoicevoxResultCode::VOICEVOX_RESULT_FAILED_GET_SUPPORTED_DEVICES
    )]
    fn convert_result_works(#[case] result: Result<()>, #[case] expected: VoicevoxResultCode) {
        let (_, actual) = convert_result(result);
        assert_eq!(expected, actual);
    }
}
