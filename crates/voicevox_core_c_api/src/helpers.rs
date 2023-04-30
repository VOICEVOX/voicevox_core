use std::collections::BTreeMap;
use std::fmt::Debug;

use thiserror::Error;

use super::*;

pub(crate) fn into_result_code_with_error(result: CApiResult<()>) -> VoicevoxResultCode {
    if let Err(err) = &result {
        display_error(err);
    }
    return into_result_code(result);

    fn display_error(err: &CApiError) {
        eprintln!("Error(Display): {err}");
        eprintln!("Error(Debug): {err:#?}");
    }

    fn into_result_code(result: CApiResult<()>) -> VoicevoxResultCode {
        use voicevox_core::{result_code::VoicevoxResultCode::*, Error::*};
        use CApiError::*;

        match result {
            Ok(()) => VOICEVOX_RESULT_OK,
            Err(RustApi(NotLoadedOpenjtalkDict)) => VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR,
            Err(RustApi(GpuSupport)) => VOICEVOX_RESULT_GPU_SUPPORT_ERROR,
            Err(RustApi(LoadModel { .. })) => VOICEVOX_RESULT_LOAD_MODEL_ERROR,
            Err(RustApi(LoadMetas(_))) => VOICEVOX_RESULT_LOAD_METAS_ERROR,
            Err(RustApi(GetSupportedDevices(_))) => VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR,
            Err(RustApi(UninitializedStatus)) => VOICEVOX_RESULT_UNINITIALIZED_STATUS_ERROR,
            Err(RustApi(InvalidSpeakerId { .. })) => VOICEVOX_RESULT_INVALID_SPEAKER_ID_ERROR,
            Err(RustApi(InvalidModelIndex { .. })) => VOICEVOX_RESULT_INVALID_MODEL_INDEX_ERROR,
            Err(RustApi(InferenceFailed)) => VOICEVOX_RESULT_INFERENCE_ERROR,
            Err(RustApi(ExtractFullContextLabel(_))) => {
                VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR
            }
            Err(RustApi(ParseKana(_))) => VOICEVOX_RESULT_PARSE_KANA_ERROR,
            Err(InvalidUtf8Input) => VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR,
            Err(InvalidAudioQuery(_)) => VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR,
        }
    }
}

type CApiResult<T> = std::result::Result<T, CApiError>;

#[derive(Error, Debug)]
pub(crate) enum CApiError {
    #[error("{0}")]
    RustApi(#[from] voicevox_core::Error),
    #[error("UTF-8として不正な入力です")]
    InvalidUtf8Input,
    #[error("無効なAudioQueryです: {0}")]
    InvalidAudioQuery(serde_json::Error),
}

pub(crate) fn create_audio_query(
    japanese_or_kana: &CStr,
    speaker_id: u32,
    method: fn(
        &mut Internal,
        &str,
        u32,
        voicevox_core::AudioQueryOptions,
    ) -> Result<AudioQueryModel>,
    options: VoicevoxAudioQueryOptions,
) -> CApiResult<CString> {
    let japanese_or_kana = ensure_utf8(japanese_or_kana)?;

    let audio_query = method(
        &mut lock_internal(),
        japanese_or_kana,
        speaker_id,
        options.into(),
    )?;
    Ok(CString::new(audio_query_model_to_json(&audio_query)).expect("should not contain '\\0'"))
}

fn audio_query_model_to_json(audio_query_model: &AudioQueryModel) -> String {
    serde_json::to_string(audio_query_model).expect("should be always valid")
}

pub(crate) fn create_accent_phrases(
    japanese_or_kana: &CStr,
    speaker_id: u32,
    method: fn(
        &mut Internal,
        &str,
        u32,
        voicevox_core::AccentPhrasesOptions,
    ) -> Result<Vec<AccentPhraseModel>>,
    options: VoicevoxAccentPhrasesOptions,
) -> CApiResult<CString> {
    let japanese_or_kana = ensure_utf8(japanese_or_kana)?;

    let accent_phrases = method(
        &mut lock_internal(),
        japanese_or_kana,
        speaker_id,
        options.into(),
    )?;
    Ok(CString::new(accent_phrases_model_to_json(&accent_phrases))
        .expect("should not contain '\\0'"))
}

fn accent_phrases_model_to_json(accent_phrases_model: &[AccentPhraseModel]) -> String {
    serde_json::to_string(accent_phrases_model).expect("should be always valid")
}

pub(crate) fn ensure_utf8(s: &CStr) -> CApiResult<&str> {
    s.to_str().map_err(|_| CApiError::InvalidUtf8Input)
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

impl From<voicevox_core::AccentPhrasesOptions> for VoicevoxAccentPhrasesOptions {
    fn from(options: voicevox_core::AccentPhrasesOptions) -> Self {
        Self { kana: options.kana }
    }
}
impl From<VoicevoxAccentPhrasesOptions> for voicevox_core::AccentPhrasesOptions {
    fn from(options: VoicevoxAccentPhrasesOptions) -> Self {
        Self { kana: options.kana }
    }
}

impl From<VoicevoxSynthesisOptions> for voicevox_core::SynthesisOptions {
    fn from(options: VoicevoxSynthesisOptions) -> Self {
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

impl From<voicevox_core::AccelerationMode> for VoicevoxAccelerationMode {
    fn from(mode: voicevox_core::AccelerationMode) -> Self {
        use voicevox_core::AccelerationMode::*;
        match mode {
            Auto => Self::VOICEVOX_ACCELERATION_MODE_AUTO,
            Cpu => Self::VOICEVOX_ACCELERATION_MODE_CPU,
            Gpu => Self::VOICEVOX_ACCELERATION_MODE_GPU,
        }
    }
}

impl From<VoicevoxAccelerationMode> for voicevox_core::AccelerationMode {
    fn from(mode: VoicevoxAccelerationMode) -> Self {
        use VoicevoxAccelerationMode::*;
        match mode {
            VOICEVOX_ACCELERATION_MODE_AUTO => Self::Auto,
            VOICEVOX_ACCELERATION_MODE_CPU => Self::Cpu,
            VOICEVOX_ACCELERATION_MODE_GPU => Self::Gpu,
        }
    }
}

impl Default for VoicevoxInitializeOptions {
    fn default() -> Self {
        let options = voicevox_core::InitializeOptions::default();
        Self {
            acceleration_mode: options.acceleration_mode.into(),
            cpu_num_threads: options.cpu_num_threads,
            load_all_models: options.load_all_models,
            open_jtalk_dict_dir: null(),
        }
    }
}

impl VoicevoxInitializeOptions {
    pub(crate) unsafe fn try_into_options(self) -> CApiResult<voicevox_core::InitializeOptions> {
        let open_jtalk_dict_dir = (!self.open_jtalk_dict_dir.is_null())
            .then(|| ensure_utf8(CStr::from_ptr(self.open_jtalk_dict_dir)).map(Into::into))
            .transpose()?;
        Ok(voicevox_core::InitializeOptions {
            acceleration_mode: self.acceleration_mode.into(),
            cpu_num_threads: self.cpu_num_threads,
            load_all_models: self.load_all_models,
            open_jtalk_dict_dir,
        })
    }
}

impl From<voicevox_core::TtsOptions> for VoicevoxTtsOptions {
    fn from(options: voicevox_core::TtsOptions) -> Self {
        Self {
            kana: options.kana,
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

impl From<VoicevoxTtsOptions> for voicevox_core::TtsOptions {
    fn from(options: VoicevoxTtsOptions) -> Self {
        Self {
            kana: options.kana,
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

impl Default for VoicevoxSynthesisOptions {
    fn default() -> Self {
        let options = voicevox_core::TtsOptions::default();
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

// privateでくくることにより，helper.rs内からもVecSizeInfoの中身の値を触れないようにする
mod private {
    pub(super) struct VecSizeInfo {
        len: usize,
        cap: usize,
    }

    impl VecSizeInfo {
        pub(super) fn new(len: usize, cap: usize) -> Self {
            Self { len, cap }
        }
        pub(super) fn len(&self) -> usize {
            self.len
        }
        pub(super) fn cap(&self) -> usize {
            self.cap
        }
    }
}

pub(crate) struct BufferManager {
    address_to_length_table: BTreeMap<usize, private::VecSizeInfo>,
}

impl BufferManager {
    pub const fn new() -> Self {
        Self {
            address_to_length_table: BTreeMap::new(),
        }
    }

    pub fn leak_vec<T>(&mut self, vec: Vec<T>) -> (*mut T, usize) {
        assert!(
            size_of::<T>() >= 1,
            "サイズが0の値のVecはコーナーケースになりやすいためエラーにする"
        );

        let len = vec.len();
        let cap = vec.capacity();
        let size_info = private::VecSizeInfo::new(len, cap);
        let ptr = vec.leak().as_ptr();
        let addr = ptr as usize;

        let not_occupied = self
            .address_to_length_table
            .insert(addr, size_info)
            .is_none();

        assert!(not_occupied, "すでに値が入っている状態はおかしい");

        (ptr as *mut T, len)
    }

    /// leak_vecでリークしたポインタをVec<T>に戻す
    /// # Safety
    /// @param buffer_ptr 必ずleak_vecで取得したポインタを設定する
    pub unsafe fn restore_vec<T>(&mut self, buffer_ptr: *const T) -> Vec<T> {
        let addr = buffer_ptr as usize;
        let size_info = self
            .address_to_length_table
            .remove(&addr)
            .expect("管理されていないポインタを渡した");

        Vec::from_raw_parts(buffer_ptr as *mut T, size_info.len(), size_info.cap())
    }

    pub fn leak_c_string(&mut self, s: CString) -> (*const c_char, usize) {
        let (ptr, size) = self.leak_vec(s.into_bytes_with_nul());

        (ptr as *const c_char, size)
    }

    /// leak_c_stringでリークしたポインタをCStringに戻す
    /// # Safety
    /// @param buffer_ptr 必ずleak_c_stringで取得したポインタを設定する
    pub unsafe fn restore_c_string(&mut self, buffer_ptr: *const c_char) -> CString {
        let vec = self.restore_vec(buffer_ptr as *const u8);
        CString::from_vec_unchecked(vec)
    }
}
