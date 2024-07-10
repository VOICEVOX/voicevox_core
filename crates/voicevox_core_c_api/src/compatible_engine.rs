use std::{
    collections::BTreeMap,
    env,
    ffi::{c_char, CString},
    sync::{Mutex, MutexGuard},
};

use libc::c_int;

use once_cell::sync::Lazy;
use voicevox_core::{StyleId, VoiceModelId, __internal::interop::PerformInference as _};

use crate::{helpers::display_error, init_logger_once};

macro_rules! ensure_initialized {
    ($synthesizer:expr $(,)?) => {
        match $synthesizer {
            Some(synthesizer) => synthesizer,
            None => {
                set_message("Statusが初期化されていません"); // 以前の`UNINITIALIZED_STATUS_ERROR`のメッセージ
                return false;
            }
        }
    };
}

static ERROR_MESSAGE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

static ONNXRUNTIME: Lazy<&'static voicevox_core::blocking::Onnxruntime> = Lazy::new(|| {
    voicevox_core::blocking::Onnxruntime::load_once()
        .exec()
        .unwrap_or_else(|err| {
            display_error(&err);
            panic!("ONNX Runtimeをロードもしくは初期化ができなかったため、クラッシュします");
        })
});

struct VoiceModelSet {
    all_vvms: Vec<voicevox_core::blocking::VoiceModel>,
    all_metas_json: CString,
    style_model_map: BTreeMap<StyleId, VoiceModelId>,
    model_map: BTreeMap<VoiceModelId, voicevox_core::blocking::VoiceModel>,
}

static VOICE_MODEL_SET: Lazy<VoiceModelSet> = Lazy::new(|| {
    let all_vvms = get_all_models();
    let model_map: BTreeMap<_, _> = all_vvms.iter().map(|vvm| (vvm.id(), vvm.clone())).collect();
    let metas = voicevox_core::__internal::interop::merge_metas(
        all_vvms.iter().flat_map(|vvm| vvm.metas()),
    );
    let mut style_model_map = BTreeMap::default();
    for vvm in all_vvms.iter() {
        for meta in vvm.metas().iter() {
            for style in meta.styles.iter() {
                style_model_map.insert(style.id, vvm.id());
            }
        }
    }

    return VoiceModelSet {
        all_metas_json: CString::new(serde_json::to_string(&metas).unwrap()).unwrap(),
        all_vvms,
        style_model_map,
        model_map,
    };

    /// # Panics
    ///
    /// 失敗したらパニックする
    fn get_all_models() -> Vec<voicevox_core::blocking::VoiceModel> {
        let root_dir = if let Some(root_dir) = env::var_os(ROOT_DIR_ENV_NAME) {
            root_dir.into()
        } else {
            process_path::get_dylib_path()
                .or_else(process_path::get_executable_path)
                .unwrap()
                .parent()
                .unwrap_or_else(|| "".as_ref())
                .join("model")
        };

        root_dir
            .read_dir()
            .and_then(|entries| entries.collect::<std::result::Result<Vec<_>, _>>())
            .unwrap_or_else(|e| panic!("{}が読めませんでした: {e}", root_dir.display()))
            .into_iter()
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "vvm"))
            .map(|entry| voicevox_core::blocking::VoiceModel::from_path(entry.path()))
            .collect::<std::result::Result<_, _>>()
            .unwrap()
    }

    const ROOT_DIR_ENV_NAME: &str = "VV_MODELS_ROOT_DIR";
});

// FIXME: この関数を消して直接`VOICE_MODEL_SET`を参照するか、あるいは`once_cell::sync::OnceCell`
// 経由でエラーをエンジンに伝達するようにする
fn voice_model_set() -> &'static VoiceModelSet {
    &VOICE_MODEL_SET
}

static SYNTHESIZER: Lazy<Mutex<Option<voicevox_core::blocking::Synthesizer<()>>>> =
    Lazy::new(|| Mutex::new(None));

fn lock_synthesizer() -> MutexGuard<'static, Option<voicevox_core::blocking::Synthesizer<()>>> {
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
    init_logger_once();
    let result = (|| {
        let synthesizer = voicevox_core::blocking::Synthesizer::new(
            *ONNXRUNTIME,
            (),
            &voicevox_core::InitializeOptions {
                acceleration_mode: if use_gpu {
                    voicevox_core::AccelerationMode::Gpu
                } else {
                    voicevox_core::AccelerationMode::Cpu
                },
                cpu_num_threads: cpu_num_threads as u16,
            },
        )?;

        if load_all_models {
            for model in &voice_model_set().all_vvms {
                synthesizer.load_voice_model(model)?;
            }
        }

        Ok::<_, voicevox_core::Error>(synthesizer)
    })();

    match result {
        Ok(synthesizer) => {
            *lock_synthesizer() = Some(synthesizer);
            true
        }
        Err(err) => {
            set_message(&format!("{err}"));
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn load_model(style_id: i64) -> bool {
    init_logger_once();
    let style_id = StyleId::new(style_id as u32);
    let model_set = voice_model_set();
    if let Some(model_id) = model_set.style_model_map.get(&style_id) {
        let vvm = model_set.model_map.get(model_id).unwrap();
        let synthesizer = &mut *lock_synthesizer();
        let result = ensure_initialized!(synthesizer).load_voice_model(vvm);
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
    init_logger_once();
    ensure_initialized!(&*lock_synthesizer())
        .is_loaded_model_by_style_id(StyleId::new(speaker_id as u32))
}

#[no_mangle]
pub extern "C" fn finalize() {
    init_logger_once();
    *lock_synthesizer() = None;
}

#[no_mangle]
pub extern "C" fn metas() -> *const c_char {
    init_logger_once();
    let model_set = voice_model_set();
    model_set.all_metas_json.as_ptr()
}

#[no_mangle]
pub extern "C" fn last_error_message() -> *const c_char {
    init_logger_once();
    ERROR_MESSAGE.lock().unwrap().as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn supported_devices() -> *const c_char {
    init_logger_once();
    return SUPPORTED_DEVICES.as_ptr();

    static SUPPORTED_DEVICES: Lazy<CString> = Lazy::new(|| {
        CString::new(
            ONNXRUNTIME
                .supported_devices()
                .unwrap()
                .to_json()
                .to_string(),
        )
        .unwrap()
    });
}

#[no_mangle]
pub extern "C" fn yukarin_s_forward(
    length: i64,
    phoneme_list: *mut i64,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    init_logger_once();
    let synthesizer = &*lock_synthesizer();
    let result = ensure_initialized!(synthesizer).predict_duration(
        unsafe { std::slice::from_raw_parts_mut(phoneme_list, length as usize) },
        StyleId::new(unsafe { *speaker_id as u32 }),
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
    init_logger_once();
    let synthesizer = &*lock_synthesizer();
    let result = ensure_initialized!(synthesizer).predict_intonation(
        length as usize,
        unsafe { std::slice::from_raw_parts(vowel_phoneme_list, length as usize) },
        unsafe { std::slice::from_raw_parts(consonant_phoneme_list, length as usize) },
        unsafe { std::slice::from_raw_parts(start_accent_list, length as usize) },
        unsafe { std::slice::from_raw_parts(end_accent_list, length as usize) },
        unsafe { std::slice::from_raw_parts(start_accent_phrase_list, length as usize) },
        unsafe { std::slice::from_raw_parts(end_accent_phrase_list, length as usize) },
        StyleId::new(unsafe { *speaker_id as u32 }),
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
    init_logger_once();
    let length = length as usize;
    let phoneme_size = phoneme_size as usize;
    let synthesizer = &*lock_synthesizer();
    let result = ensure_initialized!(synthesizer).decode(
        length,
        phoneme_size,
        unsafe { std::slice::from_raw_parts(f0, length) },
        unsafe { std::slice::from_raw_parts(phoneme, phoneme_size * length) },
        StyleId::new(unsafe { *speaker_id as u32 }),
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
