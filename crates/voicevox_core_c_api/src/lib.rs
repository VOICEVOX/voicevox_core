mod helpers;
use helpers::*;
use libc::c_void;
use once_cell::sync::Lazy;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
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

/// voicevox_coreのハードウェアアクセラレーションモードを設定する設定値
#[repr(i32)]
#[derive(Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum VoicevoxAccelerationMode {
    /// 実行環境に合った適切なハードウェアアクセラレーションモードを選択する
    VOICEVOX_ACCELERATION_MODE_AUTO = 0,
    /// ハードウェアアクセラレーションモードをCPUに設定する
    VOICEVOX_ACCELERATION_MODE_CPU = 1,
    /// ハードウェアアクセラレーションモードをGPUに設定する
    VOICEVOX_ACCELERATION_MODE_GPU = 2,
}

/// 初期化オプション
#[repr(C)]
pub struct VoicevoxInitializeOptions {
    /// ハードウェアアクセラレーションモード
    acceleration_mode: VoicevoxAccelerationMode,
    /// CPU利用数を指定
    /// 0を指定すると環境に合わせたCPUが利用される
    cpu_num_threads: u16,
    /// 全てのmodelを読み込む
    load_all_models: bool,
    /// open_jtalkの辞書ディレクトリ
    open_jtalk_dict_dir: *const c_char,
}

/// デフォルトの初期化オプションを生成する
/// @return デフォルト値が設定された初期化オプション
#[no_mangle]
pub extern "C" fn voicevox_make_default_initialize_options() -> VoicevoxInitializeOptions {
    VoicevoxInitializeOptions::default()
}

/// 初期化する
/// @param [in] options 初期化オプション
/// @return 結果コード
#[no_mangle]
pub extern "C" fn voicevox_initialize(options: VoicevoxInitializeOptions) -> VoicevoxResultCode {
    match unsafe { options.try_into_options() } {
        Ok(options) => {
            let result = lock_internal().initialize(options);
            let (_, result_code) = convert_result(result);
            result_code
        }
        Err(result_code) => result_code,
    }
}

/// modelを読み込む
/// @param [in] speaker_id 読み込むmodelのspeaker ID
/// @return 結果コード
#[no_mangle]
pub extern "C" fn voicevox_load_model(speaker_id: u32) -> VoicevoxResultCode {
    let result = lock_internal().load_model(speaker_id);
    let (_, result_code) = convert_result(result);
    result_code
}

/// ハードウェアアクセラレーションがGPUモードか判定する
/// @return GPUモードならtrueそうでないならfalse
#[no_mangle]
pub extern "C" fn voicevox_is_gpu_mode() -> bool {
    lock_internal().is_gpu_mode()
}

/// 指定したspeaker_idのmodelが読み込まれているか判定する
/// @return modelが読み込まれているのであればtrueそうでないならfalse
#[no_mangle]
pub extern "C" fn voicevox_is_model_loaded(speaker_id: u32) -> bool {
    lock_internal().is_model_loaded(speaker_id)
}

/// このライブラリの利用を終了し、確保しているリソースを開放する
#[no_mangle]
pub extern "C" fn voicevox_finalize() {
    lock_internal().finalize()
}

/// メタ情報をjsonで取得する
/// @return メタ情報のjson文字列
#[no_mangle]
pub extern "C" fn voicevox_get_metas_json() -> *const c_char {
    lock_internal().get_metas_json().as_ptr()
}

/// サポートデバイス情報をjsonで取得する
/// @return サポートデバイス情報のjson文字列
#[no_mangle]
pub extern "C" fn voicevox_get_supported_devices_json() -> *const c_char {
    lock_internal().get_supported_devices_json().as_ptr()
}

/// predict durationを実行する
/// @param [in] length phoneme_list , output のデータ長
/// @param [in] phoneme_list  音素データ
/// @param [in] speaker_id speaker ID
/// @param [out] output データの出力先
/// @return 結果コード
///
/// # Safety
/// phoneme_list は必ず length の長さだけデータがある状態で渡すこと
/// output には length で指定した長さのデータが上書きされるので length 分の領域を確保した状態で渡すこと
#[no_mangle]
pub unsafe extern "C" fn voicevox_predict_duration(
    length: usize,
    phoneme_list: *mut i64,
    speaker_id: u32,
    output: *mut f32,
) -> VoicevoxResultCode {
    let result = lock_internal().predict_duration(
        std::slice::from_raw_parts_mut(phoneme_list, length),
        speaker_id,
    );

    let (output_vec, result_code) = convert_result(result);
    if result_code == VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED {
        if let Some(output_vec) = output_vec {
            let output_slice = std::slice::from_raw_parts_mut(output, length);
            output_slice.clone_from_slice(&output_vec);
        }
    }
    result_code
}

/// predict intonationを実行する
/// @param [in] length vowel_phoneme_list , consonant_phoneme_list , start_accent_list, end_accent_list , start_accent_phrase_list , end_accent_phrase_list , output のデータ長
/// @param [in] vowel_phoneme_list 母音の音素データ
/// @param [in] consonant_phoneme_list 子音の音素データ
/// @param [in] start_accent_list 開始アクセントデータ
/// @param [in] end_accent_list 終了アクセントデータ
/// @param [in] start_accent_phrase_list 開始アクセントフレーズデータ
/// @param [in] end_accent_phrase_list 終了アクセントフレーズデータ
/// @param [in] speaker_id speaker id
/// @param [out] output データの出力先
/// @return 結果コード
///
/// # Safety
/// vowel_phoneme_list , consonant_phoneme_list , start_accent_list , end_accent_list , start_accent_phrase_list , end_accent_phrase_list は必ず length の長さだけデータがある状態で渡すこと
/// output には length で指定した長さのデータが上書きされるので length 分の領域を確保した状態で渡すこと
#[no_mangle]
pub unsafe extern "C" fn voicevox_predict_intonation(
    length: usize,
    vowel_phoneme_list: *mut i64,
    consonant_phoneme_list: *mut i64,
    start_accent_list: *mut i64,
    end_accent_list: *mut i64,
    start_accent_phrase_list: *mut i64,
    end_accent_phrase_list: *mut i64,
    speaker_id: u32,
    output: *mut f32,
) -> VoicevoxResultCode {
    let result = lock_internal().predict_intonation(
        length,
        std::slice::from_raw_parts(vowel_phoneme_list, length),
        std::slice::from_raw_parts(consonant_phoneme_list, length),
        std::slice::from_raw_parts(start_accent_list, length),
        std::slice::from_raw_parts(end_accent_list, length),
        std::slice::from_raw_parts(start_accent_phrase_list, length),
        std::slice::from_raw_parts(end_accent_phrase_list, length),
        speaker_id,
    );
    let (output_vec, result_code) = convert_result(result);
    if let Some(output_vec) = output_vec {
        let output_slice = std::slice::from_raw_parts_mut(output, length);
        output_slice.clone_from_slice(&output_vec);
    }
    result_code
}

/// decodeを実行する
/// @param [in] length f0 , output のデータ長及び phoneme のデータ長に関連する
/// @param [in] phoneme_size 音素のサイズ phoneme のデータ長に関連する
/// @param [in] f0 基本周波数
/// @param [in] phoneme 音素データ
/// @param [in] speaker_id speaker ID
/// @param [out] output データ出力先
/// @return 結果コード
///
/// # Safety
/// f0には必ず length の長さだけデータがある状態で渡すこと
/// phoneme には必ず length * phoneme_size の長さだけデータがある状態で渡すこと
/// output には length　の長さだけデータが上書きされるので length 分の領域を確保した状態で渡すこと
#[no_mangle]
pub unsafe extern "C" fn voicevox_decode(
    length: usize,
    phoneme_size: i64,
    f0: *mut f32,
    phoneme: *mut f32,
    speaker_id: u32,
    output: *mut f32,
) -> VoicevoxResultCode {
    let length = length as usize;
    let phoneme_size = phoneme_size as usize;
    let result = lock_internal().decode(
        length,
        phoneme_size,
        std::slice::from_raw_parts(f0, length),
        std::slice::from_raw_parts(phoneme, phoneme_size * length),
        speaker_id,
    );
    let (output_vec, result_code) = convert_result(result);
    if let Some(output_vec) = output_vec {
        let output_slice = std::slice::from_raw_parts_mut(output, length);
        output_slice.clone_from_slice(&output_vec);
    }
    result_code
}

/// Audio query のオプション
#[repr(C)]
pub struct VoicevoxAudioQueryOptions {
    /// aquestalk形式のkanaとしてテキストを解釈する
    kana: bool,
}

/// デフォルトの audio query のオプションを生成する
/// @return デフォルト値が設定された audio query オプション
#[no_mangle]
pub extern "C" fn voicevox_make_default_audio_query_options() -> VoicevoxAudioQueryOptions {
    voicevox_core::AudioQueryOptions::default().into()
}

/// audio query を実行する
/// @param [in] text テキスト
/// @param [in] speaker_id speaker ID
/// @param [in] options audio queryのオプション
/// @param [out] output_audio_query_json audio query を json でフォーマットしたもの
/// @return 結果コード
///
/// # Safety
/// text はnull終端文字列であること
/// output_audio_query_json は自動でheapメモリが割り当てられるので voicevox_audio_query_json_free で開放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_audio_query(
    text: *const c_char,
    speaker_id: u32,
    options: VoicevoxAudioQueryOptions,
    output_audio_query_json: *mut *mut c_char,
) -> VoicevoxResultCode {
    let text = CStr::from_ptr(text);

    let audio_query = &match create_audio_query(text, speaker_id, Internal::audio_query, options) {
        Ok(audio_query) => audio_query,
        Err(result_code) => return result_code,
    };

    write_json_to_ptr(output_audio_query_json, audio_query);
    VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED
}

/// synthesis のオプション
#[repr(C)]
pub struct VoicevoxSynthesisOptions {
    /// 疑問文の調整を有効にする
    enable_interrogative_upspeak: bool,
}

/// デフォルトの synthesis オプションを生成する
/// @return デフォルト値が設定されたsynthesisのオプション
pub extern "C" fn voicevox_make_default_synthesis_options() -> VoicevoxSynthesisOptions {
    VoicevoxSynthesisOptions::default()
}

/// synthesis を実行する
/// @param [in] audio_query_json jsonフォーマットされた audio query
/// @param [in] speaker_id  speaker ID
/// @param [in] options synthesis オプション
/// @param [out] output_wav_size 出力する wav データのサイズ
/// @param [out] output_wav wav データの出力先
/// @return 結果コード
///
/// # Safety
/// output_wav_size には出力先の領域が確保された状態でpointerに渡されていること
/// output_wav は自動で output_wav_size 分のデータが割り当てられるので voicevox_wav_free で開放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesis(
    audio_query_json: *const c_char,
    speaker_id: u32,
    options: VoicevoxSynthesisOptions,
    output_wav_size: *mut usize,
    output_wav: *mut *mut u8,
) -> VoicevoxResultCode {
    let audio_query_json = CStr::from_ptr(audio_query_json);

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

    write_wav_to_ptr(output_wav, output_wav_size, wav);
    VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED
}

/// tts オプション
#[repr(C)]
pub struct VoicevoxTtsOptions {
    /// aquestalk形式のkanaとしてテキストを解釈する
    kana: bool,
    /// 疑問文の調整を有効にする
    enable_interrogative_upspeak: bool,
}

/// デフォルトの tts オプションを生成する
/// @return tts オプション
#[no_mangle]
pub extern "C" fn voicevox_make_default_tts_options() -> VoicevoxTtsOptions {
    voicevox_core::TtsOptions::default().into()
}

/// ttsを実行する
/// @param [in] text テキスト
/// @param [in] speaker_id
/// @param [in] options tts オプション
/// @param [out] output_wav_size 出力する wav データのサイズ
/// @param [out] output_wav wav データの出力先
/// @return 結果コード
///
/// # Safety
/// output_wav_size には出力先の領域が確保された状態でpointerに渡されていること
/// output_wav は自動で output_wav_size 分のデータが割り当てられるので voicevox_wav_free で開放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_tts(
    text: *const c_char,
    speaker_id: u32,
    options: VoicevoxTtsOptions,
    output_wav_size: *mut usize,
    output_wav: *mut *mut u8,
) -> VoicevoxResultCode {
    let (output_opt, result_code) = {
        if let Ok(text) = CStr::from_ptr(text).to_str() {
            convert_result(lock_internal().tts(text, speaker_id, options.into()))
        } else {
            (None, VoicevoxResultCode::VOICEVOX_RESULT_INVALID_UTF8_INPUT)
        }
    };
    if let Some(output) = output_opt {
        write_wav_to_ptr(output_wav, output_wav_size, output.as_slice());
    }
    result_code
}

/// jsonフォーマットされた audio query データのメモリを開放する
/// @param [in] audio_query_json 開放する json フォーマットされた audio query データ
#[no_mangle]
pub extern "C" fn voicevox_audio_query_json_free(audio_query_json: *mut c_char) {
    unsafe {
        libc::free(audio_query_json as *mut c_void);
    }
}

/// wav データのメモリを開放する
/// @param [in] wav 開放する wav データ
#[no_mangle]
pub extern "C" fn voicevox_wav_free(wav: *mut u8) {
    unsafe {
        libc::free(wav as *mut c_void);
    }
}

/// エラー結果をメッセージに変換する
/// @param [in] result_code メッセージに変換する result_code
/// @return 結果コードを元に変換されたメッセージ文字列
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
