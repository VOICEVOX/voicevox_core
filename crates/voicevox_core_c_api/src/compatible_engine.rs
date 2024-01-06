use super::*;
use libc::c_int;

pub use voicevox_core::result_code::VoicevoxResultCode;

static ERROR_MESSAGE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

fn set_message(message: &str) {
    ERROR_MESSAGE
        .lock()
        .unwrap()
        .replace_range(.., &format!("{message}\0"));
}

#[no_mangle]
pub extern "C" fn initialize(use_gpu: bool, cpu_num_threads: c_int, load_all_models: bool) -> bool {
    let result = lock_internal().initialize(voicevox_core::InitializeOptions {
        acceleration_mode: if use_gpu {
            voicevox_core::AccelerationMode::Gpu
        } else {
            voicevox_core::AccelerationMode::Cpu
        },
        cpu_num_threads: cpu_num_threads as u16,
        load_all_models,
        ..Default::default()
    });
    if let Some(err) = result.err() {
        set_message(&format!("{err}"));
        false
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn load_model(speaker_id: i64) -> bool {
    let result = lock_internal().load_model(speaker_id as u32);
    if let Some(err) = result.err() {
        set_message(&format!("{err}"));
        false
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn is_model_loaded(speaker_id: i64) -> bool {
    lock_internal().is_model_loaded(speaker_id as u32)
}

#[no_mangle]
pub extern "C" fn finalize() {
    lock_internal().finalize()
}

#[no_mangle]
pub extern "C" fn metas() -> *const c_char {
    voicevox_get_metas_json()
}

#[no_mangle]
pub extern "C" fn last_error_message() -> *const c_char {
    ERROR_MESSAGE.lock().unwrap().as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn supported_devices() -> *const c_char {
    voicevox_get_supported_devices_json()
}

#[no_mangle]
pub extern "C" fn yukarin_s_forward(
    length: i64,
    phoneme_list: *mut i64,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    let result = lock_internal().predict_duration(
        unsafe { std::slice::from_raw_parts_mut(phoneme_list, length as usize) },
        unsafe { *speaker_id as u32 },
    );
    match result {
        Ok(output_vec) => {
            let output_slice = unsafe { std::slice::from_raw_parts_mut(output, length as usize) };
            output_slice.clone_from_slice(&output_vec);
            true
        }
        Err(err) => {
            set_message(&format!("{err}"));
            false
        }
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
    let result = lock_internal().predict_intonation(
        length as usize,
        unsafe { std::slice::from_raw_parts(vowel_phoneme_list, length as usize) },
        unsafe { std::slice::from_raw_parts(consonant_phoneme_list, length as usize) },
        unsafe { std::slice::from_raw_parts(start_accent_list, length as usize) },
        unsafe { std::slice::from_raw_parts(end_accent_list, length as usize) },
        unsafe { std::slice::from_raw_parts(start_accent_phrase_list, length as usize) },
        unsafe { std::slice::from_raw_parts(end_accent_phrase_list, length as usize) },
        unsafe { *speaker_id as u32 },
    );
    match result {
        Ok(output_vec) => {
            let output_slice = unsafe { std::slice::from_raw_parts_mut(output, length as usize) };
            output_slice.clone_from_slice(&output_vec);
            true
        }
        Err(err) => {
            set_message(&format!("{err}"));
            false
        }
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
    let length = length as usize;
    let phoneme_size = phoneme_size as usize;
    let result = lock_internal().decode(
        length,
        phoneme_size,
        unsafe { std::slice::from_raw_parts(f0, length) },
        unsafe { std::slice::from_raw_parts(phoneme, phoneme_size * length) },
        unsafe { *speaker_id as u32 },
    );
    match result {
        Ok(output_vec) => {
            let output_slice = unsafe { std::slice::from_raw_parts_mut(output, length * 256) };
            output_slice.clone_from_slice(&output_vec);
            true
        }
        Err(err) => {
            set_message(&format!("{err}"));
            false
        }
    }
}
