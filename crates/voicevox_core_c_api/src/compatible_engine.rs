use std::{collections::BTreeMap, sync::Arc};

use super::*;
use libc::c_int;

pub use voicevox_core::result_code::VoicevoxResultCode;
use voicevox_core::{OpenJtalk, StyleId, VoiceModel};

static ERROR_MESSAGE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

struct VoiceModelSet {
    all_metas_json: CString,
    style_model_map: BTreeMap<StyleId, VoiceModelId>,
    model_map: BTreeMap<VoiceModelId, VoiceModel>,
}

static VOICE_MODEL_SET: Lazy<Mutex<Option<VoiceModelSet>>> = Lazy::new(|| Mutex::new(None));

fn voice_model_set() -> MutexGuard<'static, Option<VoiceModelSet>> {
    let mut vm = VOICE_MODEL_SET.lock().unwrap();
    if vm.is_none() {
        let all_vvms = Handle::current()
            .block_on(VoiceModel::get_all_models())
            .unwrap();
        let model_map: BTreeMap<_, _> = all_vvms
            .iter()
            .map(|vvm| (vvm.id().clone(), vvm.clone()))
            .collect();
        let metas: Vec<_> = all_vvms.iter().flat_map(|vvm| vvm.metas()).collect();
        let mut style_model_map = BTreeMap::default();
        for vvm in all_vvms.iter() {
            for meta in vvm.metas().iter() {
                for style in meta.styles().iter() {
                    style_model_map.insert(style.id().clone(), vvm.id().clone());
                }
            }
        }

        *vm = Some(VoiceModelSet {
            all_metas_json: CString::new(serde_json::to_string(&metas).unwrap()).unwrap(),
            style_model_map,
            model_map,
        })
    }
    vm
}

static SYNTHESIZER: Lazy<Mutex<Option<voicevox_core::VoiceSynthesizer>>> =
    Lazy::new(|| Mutex::new(None));

fn lock_synthesizer() -> MutexGuard<'static, Option<voicevox_core::VoiceSynthesizer>> {
    SYNTHESIZER.lock().unwrap()
}

fn set_message(message: &str) {
    ERROR_MESSAGE
        .lock()
        .unwrap()
        .replace_range(.., &format!("{message}\0"));
}

#[no_mangle]
pub extern "C" fn initialize(use_gpu: bool, cpu_num_threads: c_int, load_all_models: bool) -> bool {
    let handle = Handle::current();
    let result = handle.block_on(voicevox_core::VoiceSynthesizer::new_with_initialize(
        Arc::new(OpenJtalk::new_without_dic()),
        &voicevox_core::InitializeOptions {
            acceleration_mode: if use_gpu {
                voicevox_core::AccelerationMode::Gpu
            } else {
                voicevox_core::AccelerationMode::Cpu
            },
            cpu_num_threads: cpu_num_threads as u16,
            load_all_models,
        },
    ));
    if let Some(err) = result.err() {
        set_message(&format!("{err}"));
        false
    } else {
        true
    }
}

#[no_mangle]
pub extern "C" fn load_model(style_id: i64) -> bool {
    let handle = Handle::current();
    let style_id = StyleId::new(style_id as u32);
    let model_set = voice_model_set();
    if let Some(model_id) = model_set.as_ref().unwrap().style_model_map.get(&style_id) {
        let vvm = model_set.as_ref().unwrap().model_map.get(model_id).unwrap();
        let mut synthesizer = lock_synthesizer();
        let result = handle.block_on(synthesizer.as_mut().unwrap().load_model(vvm));
        if let Some(err) = result.err() {
            set_message(&format!("{err}"));
            false
        } else {
            true
        }
    } else {
        set_message(&format!("{}は無効なStyle IDです", style_id));
        false
    }
}

#[no_mangle]
pub extern "C" fn is_model_loaded(speaker_id: i64) -> bool {
    lock_synthesizer()
        .as_ref()
        .unwrap()
        .is_loaded_model_by_style_id(&StyleId::new(speaker_id as u32))
}

#[no_mangle]
pub extern "C" fn finalize() {
    *lock_synthesizer() = None;
}

#[no_mangle]
pub extern "C" fn metas() -> *const c_char {
    let model_set = voice_model_set();
    model_set.as_ref().unwrap().all_metas_json.as_ptr()
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
    let synthesizer = lock_synthesizer();
    let result = Handle::current().block_on(synthesizer.as_ref().unwrap().predict_duration(
        unsafe { std::slice::from_raw_parts_mut(phoneme_list, length as usize) },
        &StyleId::new(unsafe { *speaker_id as u32 }),
    ));
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
    let synthesizer = lock_synthesizer();
    let result = Handle::current().block_on(synthesizer.as_ref().unwrap().predict_intonation(
        length as usize,
        unsafe { std::slice::from_raw_parts(vowel_phoneme_list, length as usize) },
        unsafe { std::slice::from_raw_parts(consonant_phoneme_list, length as usize) },
        unsafe { std::slice::from_raw_parts(start_accent_list, length as usize) },
        unsafe { std::slice::from_raw_parts(end_accent_list, length as usize) },
        unsafe { std::slice::from_raw_parts(start_accent_phrase_list, length as usize) },
        unsafe { std::slice::from_raw_parts(end_accent_phrase_list, length as usize) },
        &StyleId::new(unsafe { *speaker_id as u32 }),
    ));
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
    let synthesizer = lock_synthesizer();
    let result = Handle::current().block_on(synthesizer.as_ref().unwrap().decode(
        length,
        phoneme_size,
        unsafe { std::slice::from_raw_parts(f0, length) },
        unsafe { std::slice::from_raw_parts(phoneme, phoneme_size * length) },
        &StyleId::new(unsafe { *speaker_id as u32 }),
    ));
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
