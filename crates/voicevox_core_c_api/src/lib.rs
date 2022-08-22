// TODO: ドキュメントを作成する段階になったらこのallowを外し、各pointerを使用している関数にunsafeとSafety documentを追加する
#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod helpers;
use helpers::*;
use libc::c_void;
use once_cell::sync::Lazy;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::PathBuf;
use std::ptr::null;
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
 * Cの関数として公開するための型や関数を定義するこれらの実装はvoicevox_core/publish.rsに定義してある対応する関数にある
 * この関数ではvoicevox_core/publish.rsにある対応する関数の呼び出しと、その戻り値をCの形式に変換する処理のみとする
 * これはC文脈の処理と実装をわけるためと、内部実装の変更がAPIに影響を与えにくくするためである
 * voicevox_core/publish.rsにある対応する関数とはこのファイルに定義してある公開関数からvoicevoxプレフィックスを取り除いた名前の関数である
 */

pub use voicevox_core::result_code::VoicevoxResultCode;

#[repr(C)]
pub struct VoicevoxInitializeOptions {
    use_gpu: bool,
    cpu_num_threads: u16,
    load_all_models: bool,
    open_jtalk_dict_dir: *const c_char,
}

impl VoicevoxInitializeOptions {
    fn from_default_options(options: voicevox_core::InitializeOptions) -> Self {
        Self {
            use_gpu: options.use_gpu,
            cpu_num_threads: options.cpu_num_threads,
            load_all_models: options.load_all_models,
            open_jtalk_dict_dir: null(),
        }
    }

    fn try_into_options(
        self,
    ) -> std::result::Result<voicevox_core::InitializeOptions, VoicevoxResultCode> {
        let open_jtalk_dict_dir = ensure_utf8(unsafe { CStr::from_ptr(self.open_jtalk_dict_dir) })?;
        Ok(voicevox_core::InitializeOptions {
            use_gpu: self.use_gpu,
            cpu_num_threads: self.cpu_num_threads,
            load_all_models: self.load_all_models,
            open_jtalk_dict_dir: Some(PathBuf::from(open_jtalk_dict_dir)),
        })
    }
}

#[no_mangle]
pub extern "C" fn voicevox_default_initialize_options() -> VoicevoxInitializeOptions {
    VoicevoxInitializeOptions::from_default_options(voicevox_core::InitializeOptions::default())
}

#[no_mangle]
pub extern "C" fn voicevox_initialize(options: VoicevoxInitializeOptions) -> VoicevoxResultCode {
    match options.try_into_options() {
        Ok(options) => {
            let result = lock_internal().initialize(options);
            let (_, result_code) = convert_result(result);
            result_code
        }
        Err(result_code) => result_code,
    }
}

#[no_mangle]
pub extern "C" fn voicevox_load_model(speaker_id: usize) -> VoicevoxResultCode {
    let result = lock_internal().load_model(speaker_id);
    let (_, result_code) = convert_result(result);
    result_code
}

#[no_mangle]
pub extern "C" fn voicevox_is_model_loaded(speaker_id: usize) -> bool {
    lock_internal().is_model_loaded(speaker_id)
}

#[no_mangle]
pub extern "C" fn voicevox_finalize() {
    lock_internal().finalize()
}

#[no_mangle]
pub extern "C" fn voicevox_get_metas_json() -> *const c_char {
    lock_internal().get_metas_json().as_ptr()
}

#[no_mangle]
pub extern "C" fn voicevox_get_supported_devices_json() -> *const c_char {
    lock_internal().get_supported_devices_json().as_ptr()
}

#[no_mangle]
pub extern "C" fn voicevox_predict_duration(
    length: usize,
    phoneme_list: *mut i64,
    speaker_id: usize,
    output: *mut f32,
) -> VoicevoxResultCode {
    let result = lock_internal().predict_duration(
        unsafe { std::slice::from_raw_parts_mut(phoneme_list, length) },
        speaker_id,
    );

    let (output_vec, result_code) = convert_result(result);
    if result_code == VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED {
        if let Some(output_vec) = output_vec {
            let output_slice = unsafe { std::slice::from_raw_parts_mut(output, length) };
            output_slice.clone_from_slice(&output_vec);
        }
    }
    result_code
}

#[no_mangle]
pub extern "C" fn voicevox_predict_intonation(
    length: usize,
    vowel_phoneme_list: *mut i64,
    consonant_phoneme_list: *mut i64,
    start_accent_list: *mut i64,
    end_accent_list: *mut i64,
    start_accent_phrase_list: *mut i64,
    end_accent_phrase_list: *mut i64,
    speaker_id: usize,
    output: *mut f32,
) -> VoicevoxResultCode {
    let result = lock_internal().predict_intonation(
        length,
        unsafe { std::slice::from_raw_parts(vowel_phoneme_list, length) },
        unsafe { std::slice::from_raw_parts(consonant_phoneme_list, length) },
        unsafe { std::slice::from_raw_parts(start_accent_list, length) },
        unsafe { std::slice::from_raw_parts(end_accent_list, length) },
        unsafe { std::slice::from_raw_parts(start_accent_phrase_list, length) },
        unsafe { std::slice::from_raw_parts(end_accent_phrase_list, length) },
        speaker_id,
    );
    let (output_vec, result_code) = convert_result(result);
    if let Some(output_vec) = output_vec {
        let output_slice = unsafe { std::slice::from_raw_parts_mut(output, length) };
        output_slice.clone_from_slice(&output_vec);
    }
    result_code
}

#[no_mangle]
pub extern "C" fn voicevox_decode(
    length: usize,
    phoneme_size: i64,
    f0: *mut f32,
    phoneme: *mut f32,
    speaker_id: usize,
    output: *mut f32,
) -> VoicevoxResultCode {
    let length = length as usize;
    let phoneme_size = phoneme_size as usize;
    let result = lock_internal().decode(
        length,
        phoneme_size,
        unsafe { std::slice::from_raw_parts(f0, length) },
        unsafe { std::slice::from_raw_parts(phoneme, phoneme_size * length) },
        speaker_id,
    );
    let (output_vec, result_code) = convert_result(result);
    if let Some(output_vec) = output_vec {
        let output_slice = unsafe { std::slice::from_raw_parts_mut(output, length) };
        output_slice.clone_from_slice(&output_vec);
    }
    result_code
}

#[repr(C)]
pub struct VoicevoxAudioQueryOptions {
    kana: bool,
}

impl From<voicevox_core::AudioQueryOptions> for VoicevoxAudioQueryOptions {
    fn from(options: voicevox_core::AudioQueryOptions) -> Self {
        Self { kana: options.kana }
    }
}
impl From<VoicevoxAudioQueryOptions> for voicevox_core::AudioQueryOptions {
    fn from(options: VoicevoxAudioQueryOptions) -> Self {
        Self { kana: options.kana }
    }
}

#[no_mangle]
pub extern "C" fn voicevox_default_audio_query_options() -> VoicevoxAudioQueryOptions {
    voicevox_core::AudioQueryOptions::default().into()
}

#[no_mangle]
pub extern "C" fn voicevox_audio_query(
    text: *const c_char,
    speaker_id: usize,
    options: VoicevoxAudioQueryOptions,
    output_audio_query_json: *mut *mut c_char,
) -> VoicevoxResultCode {
    let text = unsafe { CStr::from_ptr(text) };

    let audio_query = &match create_audio_query(text, speaker_id, Internal::audio_query, options) {
        Ok(audio_query) => audio_query,
        Err(result_code) => return result_code,
    };

    unsafe {
        write_json_to_ptr(output_audio_query_json, audio_query);
    }
    VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED
}

#[repr(C)]
pub struct VoicevoxSynthesisOptions {}

impl From<VoicevoxSynthesisOptions> for voicevox_core::SynthesisOptions {
    fn from(_: VoicevoxSynthesisOptions) -> Self {
        Self {}
    }
}

#[no_mangle]
pub extern "C" fn voicevox_synthesis(
    audio_query_json: *const c_char,
    speaker_id: usize,
    options: VoicevoxSynthesisOptions,
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

    let (wav, result_code) =
        convert_result(lock_internal().synthesis(audio_query, speaker_id, options.into()));
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

#[repr(C)]
pub struct VoicevoxTtsOptions {
    kana: bool,
}

impl From<voicevox_core::TtsOptions> for VoicevoxTtsOptions {
    fn from(options: voicevox_core::TtsOptions) -> Self {
        Self { kana: options.kana }
    }
}

impl From<VoicevoxTtsOptions> for voicevox_core::TtsOptions {
    fn from(options: VoicevoxTtsOptions) -> Self {
        Self { kana: options.kana }
    }
}

#[no_mangle]
pub fn voicevox_default_tts_options() -> VoicevoxTtsOptions {
    voicevox_core::TtsOptions::default().into()
}

#[no_mangle]
pub extern "C" fn voicevox_tts(
    text: *const c_char,
    speaker_id: usize,
    options: VoicevoxTtsOptions,
    output_binary_size: *mut c_int,
    output_wav: *mut *mut u8,
) -> VoicevoxResultCode {
    let (output_opt, result_code) = {
        if let Ok(text) = unsafe { CStr::from_ptr(text) }.to_str() {
            convert_result(lock_internal().tts(text, speaker_id, options.into()))
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
    voicevox_core::error_result_to_message(result_code).as_ptr() as *const c_char
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
