use std::{
    collections::BTreeMap,
    env,
    ffi::{CString, c_char},
    sync::{Arc, LazyLock, Mutex, MutexGuard},
};

use libc::c_int;

use tracing::warn;
use voicevox_core::{
    __internal::interop::{PerformInference as _, ToJsonValue as _},
    StyleId, VoiceModelId,
};

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

static ERROR_MESSAGE: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

static ONNXRUNTIME: LazyLock<&'static voicevox_core::blocking::Onnxruntime> = LazyLock::new(|| {
    let alt_onnxruntime_filename = voicevox_core::blocking::Onnxruntime::LIB_VERSIONED_FILENAME
        .replace(
            voicevox_core::blocking::Onnxruntime::LIB_NAME,
            "onnxruntime",
        );
    voicevox_core::blocking::Onnxruntime::load_once()
        .perform()
        .or_else(|err| {
            warn!("{err}");
            warn!("falling back to `{alt_onnxruntime_filename}`");
            voicevox_core::blocking::Onnxruntime::load_once()
                .filename(alt_onnxruntime_filename)
                .perform()
        })
        .unwrap_or_else(|err| {
            display_error(&err);
            panic!("ONNX Runtimeをロードもしくは初期化ができなかったため、クラッシュします");
        })
});

struct VoiceModelSet {
    all_vvms: Vec<Arc<voicevox_core::blocking::VoiceModelFile>>,
    all_metas_json: CString,
    style_model_map: BTreeMap<StyleId, VoiceModelId>,
    model_map: BTreeMap<VoiceModelId, Arc<voicevox_core::blocking::VoiceModelFile>>,
}

static VOICE_MODEL_SET: LazyLock<VoiceModelSet> = LazyLock::new(|| {
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
    fn get_all_models() -> Vec<Arc<voicevox_core::blocking::VoiceModelFile>> {
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
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "vvm"))
            .map(|entry| voicevox_core::blocking::VoiceModelFile::open(entry.path()).map(Arc::new))
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

static SYNTHESIZER: LazyLock<Mutex<Option<voicevox_core::blocking::Synthesizer<()>>>> =
    LazyLock::new(|| Mutex::new(None));

fn lock_synthesizer() -> MutexGuard<'static, Option<voicevox_core::blocking::Synthesizer<()>>> {
    SYNTHESIZER.lock().unwrap()
}

fn set_message(message: &str) {
    ERROR_MESSAGE
        .lock()
        .unwrap()
        .replace_range(.., &format!("{message}\0"));
}

// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
pub extern "C" fn initialize(use_gpu: bool, cpu_num_threads: c_int, load_all_models: bool) -> bool {
    init_logger_once();
    let result = (|| {
        let synthesizer = voicevox_core::blocking::Synthesizer::builder(*ONNXRUNTIME)
            .acceleration_mode(if use_gpu {
                voicevox_core::AccelerationMode::Gpu
            } else {
                voicevox_core::AccelerationMode::Cpu
            })
            .cpu_num_threads(cpu_num_threads as u16)
            .build()?;

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

// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
pub extern "C" fn load_model(style_id: i64) -> bool {
    init_logger_once();
    let style_id = StyleId::new(style_id as u32);
    let model_set = voice_model_set();
    if let Some(model_id) = model_set.style_model_map.get(&style_id) {
        let vvm = model_set.model_map.get(model_id).unwrap();
        let synthesizer = &mut *lock_synthesizer();
        let synthesizer = ensure_initialized!(synthesizer);
        if let Err(err) = synthesizer.unload_voice_model(*model_id) {
            assert_eq!(voicevox_core::ErrorKind::ModelNotFound, err.kind());
        }
        if let Err(err) = synthesizer.load_voice_model(vvm) {
            set_message(&format!("{err}"));
            false
        } else {
            true
        }
    } else {
        set_message(&format!("{style_id}は無効なStyle IDです"));
        false
    }
}

// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
pub extern "C" fn is_model_loaded(speaker_id: i64) -> bool {
    init_logger_once();
    ensure_initialized!(&*lock_synthesizer())
        .is_loaded_model_by_style_id(StyleId::new(speaker_id as u32))
}

// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
pub extern "C" fn finalize() {
    init_logger_once();
    *lock_synthesizer() = None;
}

// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
pub extern "C" fn metas() -> *const c_char {
    init_logger_once();
    let model_set = voice_model_set();
    model_set.all_metas_json.as_ptr()
}

// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
pub extern "C" fn last_error_message() -> *const c_char {
    init_logger_once();
    ERROR_MESSAGE.lock().unwrap().as_ptr() as *const c_char
}

// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
#[unsafe(no_mangle)]
pub extern "C" fn supported_devices() -> *const c_char {
    init_logger_once();
    return SUPPORTED_DEVICES.as_ptr();

    static SUPPORTED_DEVICES: LazyLock<CString> =
        LazyLock::new(|| CString::new(ONNXRUNTIME.supported_devices().unwrap().to_json()).unwrap());
}

/// # Safety
///
/// - `phoneme_list`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `speaker_id`はRustの`&[i64; 1]`として解釈できなければならない。
/// - `output`はRustの`&mut [f32; length as usize]`として解釈できなければならない。
#[unsafe(no_mangle)] // SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
pub unsafe extern "C" fn yukarin_s_forward(
    length: i64,
    phoneme_list: *mut i64,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    init_logger_once();
    assert_aligned(phoneme_list);
    assert_aligned(speaker_id);
    assert_aligned(output);
    let synthesizer = &*lock_synthesizer();
    let result = ensure_initialized!(synthesizer).predict_duration(
        // SAFETY: The safety contract must be upheld by the caller.
        unsafe { std::slice::from_raw_parts_mut(phoneme_list, length as usize) },
        StyleId::new(unsafe { *speaker_id as u32 }),
    );
    match result {
        Ok(output_vec) => {
            // SAFETY: The safety contract must be upheld by the caller.
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

/// # Safety
///
/// - `vowel_phoneme_list`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `consonant_phoneme_list`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `start_accent_list`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `end_accent_list`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `start_accent_phrase_list`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `end_accent_phrase_list`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `speaker_id`はRustの`&[i64; 1]`として解釈できなければならない。
/// - `output`はRustの`&mut [f32; length as usize]`として解釈できなければならない。
#[unsafe(no_mangle)] // SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
pub unsafe extern "C" fn yukarin_sa_forward(
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
    assert_aligned(vowel_phoneme_list);
    assert_aligned(consonant_phoneme_list);
    assert_aligned(start_accent_list);
    assert_aligned(end_accent_list);
    assert_aligned(start_accent_phrase_list);
    assert_aligned(end_accent_phrase_list);
    assert_aligned(speaker_id);
    assert_aligned(output);
    let synthesizer = &*lock_synthesizer();
    let result = ensure_initialized!(synthesizer).predict_intonation(
        length as usize,
        // SAFETY: The safety contract must be upheld by the caller.
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
            // SAFETY: The safety contract must be upheld by the caller.
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

/// # Safety
///
/// - `f0`はRustの`&[f32; length as usize]`として解釈できなければならない。
/// - `phoneme`はRustの`&[f32; phoneme_size * length as usize]`として解釈できなければならない。
/// - `speaker_id`はRustの`&[i64; 1]`として解釈できなければならない。
/// - `output`はRustの`&mut [f32; length as usize * 256]`として解釈できなければならない。
#[unsafe(no_mangle)] // SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
pub unsafe extern "C" fn decode_forward(
    length: i64,
    phoneme_size: i64,
    f0: *mut f32,
    phoneme: *mut f32,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    init_logger_once();
    assert_aligned(f0);
    assert_aligned(phoneme);
    assert_aligned(speaker_id);
    assert_aligned(output);
    let length = length as usize;
    let phoneme_size = phoneme_size as usize;
    let synthesizer = &*lock_synthesizer();
    let result = ensure_initialized!(synthesizer).decode(
        length,
        phoneme_size,
        // SAFETY: The safety contract must be upheld by the caller.
        unsafe { std::slice::from_raw_parts(f0, length) },
        unsafe { std::slice::from_raw_parts(phoneme, phoneme_size * length) },
        StyleId::new(unsafe { *speaker_id as u32 }),
    );
    match result {
        Ok(output_vec) => {
            // SAFETY: The safety contract must be upheld by the caller.
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

/// # Safety
///
/// - `f0`はRustの`&[f32; length as usize]`として解釈できなければならない。
/// - `phoneme`はRustの`&[f32; phoneme_size * length as usize]`として解釈できなければならない。
/// - `speaker_id`はRustの`&[i64; 1]`として解釈できなければならない。
/// - `output`はRustの`&mut [MaybeUninit<f32>; ((length + 2 * 14) * 80) as usize]`として解釈できなければならない。
#[unsafe(no_mangle)] // SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
pub unsafe extern "C" fn generate_full_intermediate(
    length: i64,
    phoneme_size: i64,
    f0: *mut f32,
    phoneme: *mut f32,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    use voicevox_core::__internal::interop::MARGIN as MARGIN_WIDTH;
    const FEATURE_SIZE: usize = 80;
    init_logger_once();
    assert_aligned(f0);
    assert_aligned(phoneme);
    assert_aligned(speaker_id);
    assert_aligned(output);
    let length = length as usize;
    let phoneme_size = phoneme_size as usize;
    let synthesizer = &*lock_synthesizer();
    let result = ensure_initialized!(synthesizer).generate_full_intermediate(
        length,
        phoneme_size,
        // SAFETY: The safety contract must be upheld by the caller.
        unsafe { std::slice::from_raw_parts(f0, length) },
        unsafe { std::slice::from_raw_parts(phoneme, phoneme_size * length) },
        StyleId::new(unsafe { *speaker_id as u32 }),
    );
    match result {
        Ok(output_arr) => {
            let output_len = (length + 2 * MARGIN_WIDTH) * FEATURE_SIZE;
            if output_arr.len() != output_len {
                if output_arr.ncols() != FEATURE_SIZE {
                    panic!("the feature size is expected to be {FEATURE_SIZE}");
                } else {
                    panic!("expected {}, got {}", output_len, output_arr.len());
                }
            }
            let output_arr = output_arr.as_standard_layout();
            // SAFETY: The safety contract must be upheld by the caller.
            unsafe {
                output_arr
                    .as_ptr()
                    .copy_to_nonoverlapping(output, output_len);
            }
            true
        }
        Err(err) => {
            set_message(&format!("{err}"));
            false
        }
    }
}

/// # Safety
///
/// - `audio_feature`はRustの`&[f32; (length * feature_size) as usize]`として解釈できなければならない。
/// - `speaker_id`はRustの`&[i64; 1]`として解釈できなければならない。
/// - `output`はRustの`&mut [MaybeUninit<f32>; length as usize * 256]`として解釈できなければならない。
#[unsafe(no_mangle)] // SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
pub unsafe extern "C" fn render_audio_segment(
    length: i64,
    _margin_width: i64,
    feature_size: i64,
    audio_feature: *mut f32,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    init_logger_once();
    assert_aligned(audio_feature);
    assert_aligned(speaker_id);
    assert_aligned(output);
    let length = length as usize;
    let feature_size = feature_size as usize;
    let synthesizer = &*lock_synthesizer();
    let result = ensure_initialized!(synthesizer).render_audio_segment(
        // SAFETY: The safety contract must be upheld by the caller.
        unsafe {
            ndarray::ArrayView2::from_shape_ptr([length, feature_size], audio_feature).to_owned()
        },
        StyleId::new(unsafe { *speaker_id as u32 }),
    );
    match result {
        Ok(output_arr) => {
            let output_len = length * 256;
            if output_arr.len() != output_len {
                panic!("expected {}, got {}", output_len, output_arr.len());
            }
            let output_arr = output_arr.as_standard_layout();
            // SAFETY: The safety contract must be upheld by the caller.
            unsafe {
                output_arr
                    .as_ptr()
                    .copy_to_nonoverlapping(output, output_len);
            }
            true
        }
        Err(err) => {
            set_message(&format!("{err}"));
            false
        }
    }
}

/// # Safety
///
/// - `consonant`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `vowel`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `note_duration`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `speaker_id`はRustの`&[i64; 1]`として解釈できなければならない。
/// - `output`はRustの`&mut [MaybeUninit<i64>; length as usize]`として解釈できなければならない。
#[unsafe(no_mangle)] // SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
pub unsafe extern "C" fn predict_sing_consonant_length_forward(
    length: i64,
    consonant: *mut i64,
    vowel: *mut i64,
    note_duration: *mut i64,
    speaker_id: *mut i64,
    output: *mut i64,
) -> bool {
    init_logger_once();
    assert_aligned(consonant);
    assert_aligned(vowel);
    assert_aligned(note_duration);
    assert_aligned(speaker_id);
    assert_aligned(output);
    let length = length as usize;
    let synthesizer = &*lock_synthesizer();
    let result = ensure_initialized!(synthesizer).predict_sing_consonant_length(
        // SAFETY: The safety contract must be upheld by the caller.
        unsafe { ndarray::ArrayView::from_shape_ptr([length], consonant) }.to_owned(),
        unsafe { ndarray::ArrayView::from_shape_ptr([length], vowel) }.to_owned(),
        unsafe { ndarray::ArrayView::from_shape_ptr([length], note_duration) }.to_owned(),
        StyleId::new(unsafe { *speaker_id as u32 }),
    );
    match result {
        Ok(output_arr) => {
            let output_len = length;
            if output_arr.len() != output_len {
                panic!("expected {}, got {}", output_len, output_arr.len());
            }
            let output_arr = output_arr.as_standard_layout();
            // SAFETY: The safety contract must be upheld by the caller.
            unsafe {
                output_arr
                    .as_ptr()
                    .copy_to_nonoverlapping(output, output_len);
            }
            true
        }
        Err(err) => {
            set_message(&format!("{err}"));
            false
        }
    }
}

/// # Safety
///
/// - `phoneme`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `note`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `speaker_id`はRustの`&[i64; 1]`として解釈できなければならない。
/// - `output`はRustの`&mut [MaybeUninit<f32>; length as usize]`として解釈できなければならない。
#[unsafe(no_mangle)] // SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
pub unsafe extern "C" fn predict_sing_f0_forward(
    length: i64,
    phoneme: *mut i64,
    note: *mut i64,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    init_logger_once();
    assert_aligned(phoneme);
    assert_aligned(note);
    assert_aligned(speaker_id);
    assert_aligned(output);
    let length = length as usize;
    let synthesizer = &*lock_synthesizer();
    let result = ensure_initialized!(synthesizer).predict_sing_f0(
        // SAFETY: The safety contract must be upheld by the caller.
        unsafe { ndarray::ArrayView::from_shape_ptr([length], phoneme) }.to_owned(),
        unsafe { ndarray::ArrayView::from_shape_ptr([length], note) }.to_owned(),
        StyleId::new(unsafe { *speaker_id as u32 }),
    );
    match result {
        Ok(output_arr) => {
            let output_len = length;
            if output_arr.len() != output_len {
                panic!("expected {}, got {}", output_len, output_arr.len());
            }
            let output_arr = output_arr.as_standard_layout();
            // SAFETY: The safety contract must be upheld by the caller.
            unsafe {
                output_arr
                    .as_ptr()
                    .copy_to_nonoverlapping(output, output_len);
            }
            true
        }
        Err(err) => {
            set_message(&format!("{err}"));
            false
        }
    }
}

/// # Safety
///
/// - `phoneme`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `note`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `f0`はRustの`&[f32; length as usize]`として解釈できなければならない。
/// - `speaker_id`はRustの`&[i64; 1]`として解釈できなければならない。
/// - `output`はRustの`&mut [MaybeUninit<f32>; length as usize]`として解釈できなければならない。
#[unsafe(no_mangle)] // SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
pub unsafe extern "C" fn predict_sing_volume_forward(
    length: i64,
    phoneme: *mut i64,
    note: *mut i64,
    f0: *mut f32,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    init_logger_once();
    assert_aligned(phoneme);
    assert_aligned(note);
    assert_aligned(f0);
    assert_aligned(speaker_id);
    assert_aligned(output);
    let length = length as usize;
    let synthesizer = &*lock_synthesizer();
    let result = ensure_initialized!(synthesizer).predict_sing_volume(
        // SAFETY: The safety contract must be upheld by the caller.
        unsafe { ndarray::ArrayView::from_shape_ptr([length], phoneme) }.to_owned(),
        unsafe { ndarray::ArrayView::from_shape_ptr([length], note) }.to_owned(),
        unsafe { ndarray::ArrayView::from_shape_ptr([length], f0) }.to_owned(),
        StyleId::new(unsafe { *speaker_id as u32 }),
    );
    match result {
        Ok(output_arr) => {
            let output_len = length;
            if output_arr.len() != output_len {
                panic!("expected {}, got {}", output_len, output_arr.len());
            }
            let output_arr = output_arr.as_standard_layout();
            // SAFETY: The safety contract must be upheld by the caller.
            unsafe {
                output_arr
                    .as_ptr()
                    .copy_to_nonoverlapping(output, output_len);
            }
            true
        }
        Err(err) => {
            set_message(&format!("{err}"));
            false
        }
    }
}

/// # Safety
///
/// - `phoneme`はRustの`&[i64; length as usize]`として解釈できなければならない。
/// - `f0`はRustの`&[f32; length as usize]`として解釈できなければならない。
/// - `volume`はRustの`&[f32; length as usize]`として解釈できなければならない。
/// - `speaker_id`はRustの`&[i64; 1]`として解釈できなければならない。
/// - `output`はRustの`&mut [MaybeUninit<f32>; length as usize]`として解釈できなければならない。
#[unsafe(no_mangle)] // SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
pub unsafe extern "C" fn sf_decode_forward(
    length: i64,
    phoneme: *mut i64,
    f0: *mut f32,
    volume: *mut f32,
    speaker_id: *mut i64,
    output: *mut f32,
) -> bool {
    init_logger_once();
    assert_aligned(phoneme);
    assert_aligned(f0);
    assert_aligned(volume);
    assert_aligned(speaker_id);
    assert_aligned(output);
    let length = length as usize;
    let synthesizer = &*lock_synthesizer();
    let result = ensure_initialized!(synthesizer).sf_decode(
        // SAFETY: The safety contract must be upheld by the caller.
        unsafe { ndarray::ArrayView::from_shape_ptr([length], phoneme) }.to_owned(),
        unsafe { ndarray::ArrayView::from_shape_ptr([length], f0) }.to_owned(),
        unsafe { ndarray::ArrayView::from_shape_ptr([length], volume) }.to_owned(),
        StyleId::new(unsafe { *speaker_id as u32 }),
    );
    match result {
        Ok(output_arr) => {
            let output_len = length * 256;
            if output_arr.len() != output_len {
                panic!("expected {}, got {}", output_len, output_arr.len());
            }
            let output_arr = output_arr.as_standard_layout();
            // SAFETY: The safety contract must be upheld by the caller.
            unsafe {
                output_arr
                    .as_ptr()
                    .copy_to_nonoverlapping(output, output_len);
            }
            true
        }
        Err(err) => {
            set_message(&format!("{err}"));
            false
        }
    }
}

#[track_caller]
fn assert_aligned(ptr: *mut impl Sized) {
    assert!(
        ptr.is_aligned(),
        "all of the pointers passed to this library **must** be aligned",
    );
}
