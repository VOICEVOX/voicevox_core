use std::ffi::{c_char, c_int, c_void};

use libloading::{Library, Symbol};
use voicevox_core::result_code::VoicevoxResultCode;

/// voicevox\_core\_c\_apiのcdylibのシンボルを集めたもの。
#[allow(dead_code)] // TODO: WIP
pub(crate) struct Symbols<'lib> {
    pub(crate) voicevox_open_jtalk_rc_new: Symbol<
        'lib,
        unsafe extern "C" fn(*const c_char, *mut *mut OpenJtalkRc) -> VoicevoxResultCode,
    >,
    pub(crate) voicevox_open_jtalk_rc_delete: Symbol<'lib, unsafe extern "C" fn(*mut OpenJtalkRc)>,
    pub(crate) voicevox_make_default_initialize_options:
        Symbol<'lib, unsafe extern "C" fn() -> VoicevoxInitializeOptions>,
    pub(crate) voicevox_get_version: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    pub(crate) voicevox_voice_model_new_from_path: Symbol<
        'lib,
        unsafe extern "C" fn(*const c_char, *mut *mut VoicevoxVoiceModel) -> VoicevoxResultCode,
    >,
    pub(crate) voicevox_voice_model_id:
        Symbol<'lib, unsafe extern "C" fn(*const VoicevoxVoiceModel) -> VoicevoxVoiceModelId>,
    pub(crate) voicevox_voice_model_get_metas_json:
        Symbol<'lib, unsafe extern "C" fn(*const VoicevoxVoiceModel) -> *const c_char>,
    pub(crate) voicevox_voice_model_delete:
        Symbol<'lib, unsafe extern "C" fn(*mut VoicevoxVoiceModel)>,
    pub(crate) voicevox_synthesizer_new_with_initialize: Symbol<
        'lib,
        unsafe extern "C" fn(
            *const OpenJtalkRc,
            VoicevoxInitializeOptions,
            *mut *mut VoicevoxSynthesizer,
        ) -> VoicevoxResultCode,
    >,
    pub(crate) voicevox_synthesizer_delete:
        Symbol<'lib, unsafe extern "C" fn(*mut VoicevoxSynthesizer)>,
    pub(crate) voicevox_synthesizer_load_voice_model: Symbol<
        'lib,
        unsafe extern "C" fn(
            *mut VoicevoxSynthesizer,
            *const VoicevoxVoiceModel,
        ) -> VoicevoxResultCode,
    >,
    pub(crate) voicevox_synthesizer_unload_voice_model: Symbol<
        'lib,
        unsafe extern "C" fn(*mut VoicevoxSynthesizer, VoicevoxVoiceModelId) -> VoicevoxResultCode,
    >,
    pub(crate) voicevox_synthesizer_is_gpu_mode:
        Symbol<'lib, unsafe extern "C" fn(*const VoicevoxSynthesizer) -> bool>,
    pub(crate) voicevox_is_loaded_voice_model: Symbol<
        'lib,
        unsafe extern "C" fn(*const VoicevoxSynthesizer, VoicevoxVoiceModelId) -> bool,
    >,
    pub(crate) voicevox_synthesizer_get_metas_json:
        Symbol<'lib, unsafe extern "C" fn(*const VoicevoxSynthesizer) -> *const c_char>,
    pub(crate) voicevox_create_supported_devices_json:
        Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    pub(crate) voicevox_make_default_audio_query_options:
        Symbol<'lib, unsafe extern "C" fn() -> VoicevoxAudioQueryOptions>,
    pub(crate) voicevox_synthesizer_audio_query: Symbol<
        'lib,
        unsafe extern "C" fn(
            *const VoicevoxSynthesizer,
            *const c_char,
            VoicevoxStyleId,
            VoicevoxAudioQueryOptions,
            *mut *mut c_char,
        ) -> VoicevoxResultCode,
    >,
    pub(crate) voicevox_make_default_synthesis_options:
        Symbol<'lib, unsafe extern "C" fn() -> VoicevoxSynthesisOptions>,
    pub(crate) voicevox_synthesizer_synthesis: Symbol<
        'lib,
        unsafe extern "C" fn(
            *const VoicevoxSynthesizer,
            *const c_char,
            VoicevoxStyleId,
            VoicevoxSynthesisOptions,
            *mut usize,
            *mut *mut u8,
        ) -> VoicevoxResultCode,
    >,
    pub(crate) voicevox_make_default_tts_options:
        Symbol<'lib, unsafe extern "C" fn() -> VoicevoxTtsOptions>,
    pub(crate) voicevox_synthesizer_tts: Symbol<
        'lib,
        unsafe extern "C" fn(
            *const VoicevoxSynthesizer,
            *const c_char,
            VoicevoxStyleId,
            VoicevoxTtsOptions,
            *mut usize,
            *mut *mut u8,
        ) -> VoicevoxResultCode,
    >,
    pub(crate) voicevox_json_free: Symbol<'lib, unsafe extern "C" fn(*mut c_char)>,
    pub(crate) voicevox_wav_free: Symbol<'lib, unsafe extern "C" fn(*mut u8)>,
    pub(crate) voicevox_error_result_to_message:
        Symbol<'lib, unsafe extern "C" fn(VoicevoxResultCode) -> *const c_char>,

    pub(crate) initialize: Symbol<'lib, unsafe extern "C" fn(bool, c_int, bool) -> bool>,
    pub(crate) load_model: Symbol<'lib, unsafe extern "C" fn(i64) -> bool>,
    pub(crate) is_model_loaded: Symbol<'lib, unsafe extern "C" fn(i64) -> bool>,
    pub(crate) finalize: Symbol<'lib, unsafe extern "C" fn()>,
    pub(crate) metas: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    pub(crate) last_error_message: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    pub(crate) supported_devices: Symbol<'lib, unsafe extern "C" fn() -> *const c_char>,
    pub(crate) yukarin_s_forward:
        Symbol<'lib, unsafe extern "C" fn(i64, *mut i64, *mut i64, *mut f32) -> bool>,
    pub(crate) yukarin_sa_forward: Symbol<
        'lib,
        unsafe extern "C" fn(
            i64,
            *mut i64,
            *mut i64,
            *mut i64,
            *mut i64,
            *mut i64,
            *mut i64,
            *mut i64,
            *mut f32,
        ) -> bool,
    >,
    pub(crate) decode_forward: Symbol<
        'lib,
        unsafe extern "C" fn(i64, i64, *mut f32, *mut f32, *mut i64, *mut f32) -> bool,
    >,
}

impl<'lib> Symbols<'lib> {
    pub(crate) unsafe fn new(lib: &'lib Library) -> Result<Self, libloading::Error> {
        macro_rules! new(($($name:ident),* $(,)?) => {
            Self {
                $(
                    $name: lib.get(stringify!($name).as_ref())?,
                )*
            }
        });

        Ok(new!(
            voicevox_open_jtalk_rc_new,
            voicevox_open_jtalk_rc_delete,
            voicevox_make_default_initialize_options,
            voicevox_get_version,
            voicevox_voice_model_new_from_path,
            voicevox_voice_model_id,
            voicevox_voice_model_get_metas_json,
            voicevox_voice_model_delete,
            voicevox_synthesizer_new_with_initialize,
            voicevox_synthesizer_delete,
            voicevox_synthesizer_load_voice_model,
            voicevox_synthesizer_unload_voice_model,
            voicevox_synthesizer_is_gpu_mode,
            voicevox_is_loaded_voice_model,
            voicevox_synthesizer_get_metas_json,
            voicevox_create_supported_devices_json,
            voicevox_make_default_audio_query_options,
            voicevox_synthesizer_audio_query,
            voicevox_make_default_synthesis_options,
            voicevox_synthesizer_synthesis,
            voicevox_make_default_tts_options,
            voicevox_synthesizer_tts,
            voicevox_json_free,
            voicevox_wav_free,
            voicevox_error_result_to_message,
            initialize,
            load_model,
            is_model_loaded,
            finalize,
            metas,
            last_error_message,
            supported_devices,
            yukarin_s_forward,
            yukarin_sa_forward,
            decode_forward,
        ))
    }
}

type OpenJtalkRc = c_void;
type VoicevoxVoiceModel = c_void;
type VoicevoxVoiceModelId = *const c_char;
type VoicevoxSynthesizer = c_void;
type VoicevoxStyleId = u32;

#[repr(i32)]
#[allow(non_camel_case_types)]
pub(crate) enum VoicevoxAccelerationMode {
    VOICEVOX_ACCELERATION_MODE_CPU = 1,
}

#[repr(C)]
pub(crate) struct VoicevoxInitializeOptions {
    pub(crate) acceleration_mode: VoicevoxAccelerationMode,
    pub(crate) _cpu_num_threads: u16,
    pub(crate) _load_all_models: bool,
}

#[repr(C)]
pub(crate) struct VoicevoxAudioQueryOptions {
    _kana: bool,
}

#[repr(C)]
pub(crate) struct VoicevoxSynthesisOptions {
    _enable_interrogative_upspeak: bool,
}

#[repr(C)]
pub(crate) struct VoicevoxTtsOptions {
    _kana: bool,
    _enable_interrogative_upspeak: bool,
}
