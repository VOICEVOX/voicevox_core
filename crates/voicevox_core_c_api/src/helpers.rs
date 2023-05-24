use std::alloc::Layout;
use std::collections::{BTreeMap, BTreeSet, HashSet};
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
            Err(InvalidAccentPhrase(_)) => VOICEVOX_RESULT_INVALID_ACCENT_PHRASE_ERROR,
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
    #[error("無効なAccentPhraseです: {0}")]
    InvalidAccentPhrase(serde_json::Error),
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

pub(crate) fn modify_accent_phrases(
    accent_phrases: &[AccentPhraseModel],
    speaker_id: u32,
    method: fn(&mut Internal, u32, &[AccentPhraseModel]) -> Result<Vec<AccentPhraseModel>>,
) -> CApiResult<CString> {
    let accent_phrases = method(&mut lock_internal(), speaker_id, accent_phrases)?;
    Ok(CString::new(accent_phrases_model_to_json(&accent_phrases))
        .expect("should not contain '\\0'"))
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

// libcのmallocで追加のアロケーションを行うことなく、`Vec<u8>`や`Vec<f32>`の内容を直接Cの世界に貸し出す。

/// Rustの世界の`Box<[impl Copy]>`をCの世界に貸し出すため、アドレスとレイアウトを管理するもの。
pub(crate) struct BufferManager {
    address_to_layout_table: BTreeMap<usize, Layout>,
    json_addrs: BTreeSet<usize>,
    static_str_addrs: fn() -> HashSet<usize>,
}

impl BufferManager {
    pub fn new(static_str_addrs: fn() -> HashSet<usize>) -> Self {
        Self {
            address_to_layout_table: BTreeMap::new(),
            json_addrs: BTreeSet::new(),
            static_str_addrs,
        }
    }

    pub fn vec_into_raw<T: Copy>(&mut self, vec: Vec<T>) -> (*mut T, usize) {
        let slice = Box::leak(vec.into_boxed_slice());
        let layout = Layout::for_value(slice);
        let len = slice.len();
        let ptr = slice.as_mut_ptr();
        let addr = ptr as usize;

        let not_occupied = self.address_to_layout_table.insert(addr, layout).is_none();

        assert!(not_occupied, "すでに値が入っている状態はおかしい");

        (ptr, len)
    }

    /// `vec_into_raw`でC API利用側に貸し出したポインタに対し、デアロケートする。
    ///
    /// # Safety
    ///
    /// - `buffer_ptr`は`vec_into_raw`で取得したものであること。
    pub unsafe fn dealloc_slice<T: Copy>(&mut self, buffer_ptr: *const T) {
        let addr = buffer_ptr as usize;
        let layout = self.address_to_layout_table.remove(&addr).expect(
            "解放しようとしたポインタはvoicevox_coreの管理下にありません。\
             誤ったポインタであるか、二重解放になっていることが考えられます",
        );

        if layout.size() > 0 {
            // `T: Copy`より、`T: !Drop`であるため`drop_in_place`は不要

            // SAFETY:
            // - `addr`と`layout`は対応したものである
            // - `layout.size() > 0`より、`addr`はダングリングではない有効なポインタである
            std::alloc::dealloc(addr as *mut u8, layout);
        }
    }

    pub fn c_string_into_raw(&mut self, s: CString) -> *mut c_char {
        let ptr = s.into_raw();
        self.json_addrs.insert(ptr as _);
        ptr
    }

    /// `c_string_into_raw`でC API利用側に貸し出したポインタに対し、デアロケートする。
    ///
    /// # Safety
    ///
    /// - `ptr`は`c_string_into_raw`で取得したものであること。
    pub unsafe fn dealloc_c_string(&mut self, ptr: *mut c_char) {
        if !self.json_addrs.remove(&(ptr as _)) {
            if (self.static_str_addrs)().contains(&(ptr as _)) {
                panic!(
                    "解放しようとしたポインタはvoicevox_core管理下のものですが、\
                     voicevox_coreがアンロードされるまで永続する文字列に対するものです。\
                     解放することはできません",
                )
            }
            panic!(
                "解放しようとしたポインタはvoicevox_coreの管理下にありません。\
                 誤ったポインタであるか、二重解放になっていることが考えられます",
            );
        }
        drop(CString::from_raw(ptr));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_manager_works() {
        let mut buffer_manager = BufferManager::new(|| unreachable!());

        rent_and_dealloc(&mut buffer_manager, vec::<()>(0, &[]));
        rent_and_dealloc(&mut buffer_manager, vec(0, &[()]));
        rent_and_dealloc(&mut buffer_manager, vec(2, &[()]));

        rent_and_dealloc(&mut buffer_manager, vec::<u8>(0, &[]));
        rent_and_dealloc(&mut buffer_manager, vec(0, &[0u8]));
        rent_and_dealloc(&mut buffer_manager, vec(2, &[0u8]));

        rent_and_dealloc(&mut buffer_manager, vec::<f32>(0, &[]));
        rent_and_dealloc(&mut buffer_manager, vec(0, &[0f32]));
        rent_and_dealloc(&mut buffer_manager, vec(2, &[0f32]));

        fn rent_and_dealloc(buffer_manager: &mut BufferManager, vec: Vec<impl Copy>) {
            let expected_len = vec.len();
            let (ptr, len) = buffer_manager.vec_into_raw(vec);
            assert_eq!(expected_len, len);
            unsafe {
                buffer_manager.dealloc_slice(ptr);
            }
        }

        fn vec<T: Copy>(initial_cap: usize, elems: &[T]) -> Vec<T> {
            let mut vec = Vec::with_capacity(initial_cap);
            vec.extend_from_slice(elems);
            vec
        }
    }

    #[test]
    #[should_panic(
        expected = "解放しようとしたポインタはvoicevox_coreの管理下にありません。誤ったポインタであるか、二重解放になっていることが考えられます"
    )]
    fn buffer_manager_denies_unknown_slice_ptr() {
        let mut buffer_manager = BufferManager::new(|| [].into());
        unsafe {
            let x = 42;
            buffer_manager.dealloc_slice(&x as *const i32);
        }
    }

    #[test]
    #[should_panic(
        expected = "解放しようとしたポインタはvoicevox_coreの管理下にありません。誤ったポインタであるか、二重解放になっていることが考えられます"
    )]
    fn buffer_manager_denies_unknown_char_ptr() {
        let mut buffer_manager = BufferManager::new(|| [].into());
        unsafe {
            let s = CStr::from_bytes_with_nul(b"\0").unwrap().to_owned();
            buffer_manager.dealloc_c_string(s.into_raw());
        }
    }

    #[test]
    #[should_panic(
        expected = "解放しようとしたポインタはvoicevox_core管理下のものですが、voicevox_coreがアンロードされるまで永続する文字列に対するものです。解放することはできません"
    )]
    fn buffer_manager_denies_known_static_char_ptr() {
        let mut buffer_manager = BufferManager::new(|| [STATIC.as_ptr() as _].into());
        unsafe {
            buffer_manager.dealloc_c_string(STATIC.as_ptr() as *mut c_char);
        }

        static STATIC: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"\0") };
    }
}
