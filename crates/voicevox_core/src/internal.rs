use super::*;
use c_export::VoicevoxResultCode;
use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::os::raw::c_int;
use std::sync::Mutex;

use status::*;

static INITIALIZED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static STATUS: Lazy<Mutex<Option<Status>>> = Lazy::new(|| Mutex::new(None));

pub fn initialize(use_gpu: bool, cpu_num_threads: usize, load_all_models: bool) -> Result<()> {
    let mut initialized = INITIALIZED.lock().unwrap();
    *initialized = false;
    if !use_gpu || can_support_gpu_feature()? {
        let mut status_opt = STATUS.lock().unwrap();
        let mut status = Status::new(use_gpu, cpu_num_threads);

        // TODO: ここに status.load_metas() を呼び出すようにする
        // https://github.com/VOICEVOX/voicevox_core/blob/main/core/src/core.cpp#L199-L201

        if load_all_models {
            for model_index in 0..Status::MODELS_COUNT {
                status.load_model(model_index)?;
            }
            // TODO: ここにGPUメモリを確保させる処理を実装する
            // https://github.com/VOICEVOX/voicevox_core/blob/main/core/src/core.cpp#L210-L219
        }

        *status_opt = Some(status);
        *initialized = true;
        Ok(())
    } else {
        Err(Error::CantGpuSupport)
    }
}

fn can_support_gpu_feature() -> Result<bool> {
    let supported_devices = SupportedDevices::get_supported_devices()?;

    cfg_if! {
        if #[cfg(feature = "directml")]{
            Ok(*supported_devices.dml())
        } else{
            Ok(*supported_devices.cuda())
        }
    }
}

//TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
#[allow(unused_variables)]
pub fn load_model(speaker_id: i64) -> Result<()> {
    unimplemented!()
}

//TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
#[allow(unused_variables)]
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

//TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
#[allow(unused_variables)]
pub fn yukarin_s_forward(
    length: i64,
    phoneme_list: *const i64,
    speaker_id: &i64,
    output: *mut f32,
) -> Result<()> {
    unimplemented!()
}

//TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
#[allow(unused_variables)]
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

//TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
#[allow(unused_variables)]
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

//TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
#[allow(unused_variables)]
pub fn voicevox_load_openjtalk_dict(dict_path: &CStr) -> Result<()> {
    unimplemented!()
}

//TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
#[allow(unused_variables)]
pub fn voicevox_tts(
    text: &CStr,
    speaker_id: i64,
    output_binary_size: *mut c_int,
    output_wav: *const *mut u8,
) -> Result<()> {
    unimplemented!()
}

//TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
#[allow(unused_variables)]
pub fn voicevox_tts_from_kana(
    text: &CStr,
    speaker_id: i64,
    output_binary_size: *mut c_int,
    output_wav: *const *mut u8,
) -> Result<()> {
    unimplemented!()
}

//TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
#[allow(unused_variables)]
pub fn voicevox_wav_free(wav: *mut u8) -> Result<()> {
    unimplemented!()
}

pub const fn voicevox_error_result_to_message(result_code: VoicevoxResultCode) -> &'static str {
    // C APIのため、messageには必ず末尾にNULL文字を追加する
    use VoicevoxResultCode::*;
    match result_code {
        VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT => {
            "voicevox_load_openjtalk_dict() を初めに呼んでください\0"
        }
        VOICEVOX_RESULT_FAILED_LOAD_MODEL => {
            "modelデータ読み込み中にOnnxruntimeエラーが発生しました\0"
        }

        VOICEVOX_RESULT_CANT_GPU_SUPPORT => "GPU機能をサポートすることができません\0",
        VOICEVOX_RESULT_FAILED_GET_SUPPORTED_DEVICES => {
            "サポートされているデバイス情報取得中にエラーが発生しました\0"
        }

        VOICEVOX_RESULT_SUCCEED => "エラーが発生しませんでした\0",
    }
}
