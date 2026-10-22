use std::{
    cmp,
    collections::{HashMap, HashSet},
    ffi::CString,
    num::NonZero,
    path::Path,
    ptr::NonNull,
    sync::{Arc, LazyLock},
};

use camino::Utf8Path;
use duplicate::duplicate_item;
use easy_ext::ext;
use ref_cast::ref_cast_custom;
use voicevox_core::{
    __internal::interop::{AudioFeatureOutOfBoundsError, AudioFeatureSliceExt as _},
    AudioFeatureSlice, CharacterMeta, Result, VoiceModelId,
};

use crate::{
    OpenJtalkRc, VoicevoxAudioFeature, VoicevoxAudioFeatureSlice, VoicevoxInitializeOptions,
    VoicevoxOnnxruntime, VoicevoxSynthesizer, VoicevoxUserDict, VoicevoxVoiceModelFile,
    helpers::{CApiError, CApiResult},
    object::{CApiObject, CApiObjectPtrExt as _, MaybeDeleted},
};

// FIXME: 中身(Rust API)を直接操作するかラッパーメソッド越しにするのかが混在していて、一貫性を
// 欠いている

impl VoicevoxOnnxruntime {
    pub(crate) const LIB_MIN_REQUIRED_MINOR_VERSION: u32 =
        voicevox_core::blocking::Onnxruntime::LIB_MIN_REQUIRED_MINOR_VERSION;

    pub(crate) const LIB_MAX_SUPPORTED_MINOR_VERSION: u32 =
        voicevox_core::blocking::Onnxruntime::LIB_MAX_SUPPORTED_MINOR_VERSION;

    #[cfg(feature = "load-onnxruntime")]
    pub(crate) const LIB_RECOMMENDED_VERSIONED_FILENAME: &'static std::ffi::CStr =
        to_cstr!(voicevox_core::blocking::Onnxruntime::LIB_RECOMMENDED_VERSIONED_FILENAME);

    #[cfg(feature = "load-onnxruntime")]
    pub(crate) const LIB_RECOMMENDED_UNVERSIONED_FILENAME: &'static std::ffi::CStr =
        to_cstr!(voicevox_core::blocking::Onnxruntime::LIB_RECOMMENDED_UNVERSIONED_FILENAME);

    #[ref_cast_custom]
    fn new(rust: &voicevox_core::blocking::Onnxruntime) -> &Self;

    pub(crate) fn get() -> Option<&'static Self> {
        voicevox_core::blocking::Onnxruntime::get().map(Self::new)
    }

    #[cfg(feature = "load-onnxruntime")]
    pub(crate) fn load_once(filename: &std::ffi::CStr) -> CApiResult<&'static Self> {
        use crate::helpers::ensure_utf8;

        let inner = voicevox_core::blocking::Onnxruntime::load_once()
            .filename(ensure_utf8(filename)?)
            .perform()?;
        Ok(Self::new(inner))
    }

    #[cfg(feature = "link-onnxruntime")]
    pub(crate) fn init_once() -> CApiResult<&'static Self> {
        let inner = voicevox_core::blocking::Onnxruntime::init_once()?;
        Ok(Self::new(inner))
    }
}

#[cfg(feature = "load-onnxruntime")]
macro_rules! to_cstr {
    ($s:expr) => {{
        static CSTR: &[u8] = const_format::concatcp!($s, '\0').as_bytes();
        unsafe {
            // SAFETY: added a nul with `concatcp!`
            std::ffi::CStr::from_bytes_with_nul_unchecked(CSTR)
        }
    }};
}
#[cfg(feature = "load-onnxruntime")]
use to_cstr;

impl OpenJtalkRc {
    pub(crate) fn new(open_jtalk_dic_dir: impl AsRef<Utf8Path>) -> Result<NonNull<Self>> {
        let body = voicevox_core::blocking::OpenJtalk::new(open_jtalk_dic_dir)?;
        Ok(<Self as CApiObject>::new(body))
    }
}

impl VoicevoxSynthesizer {
    pub(crate) fn new(
        onnxruntime: &'static VoicevoxOnnxruntime,
        open_jtalk: *const OpenJtalkRc,
        VoicevoxInitializeOptions {
            acceleration_mode,
            cpu_num_threads,
        }: VoicevoxInitializeOptions,
    ) -> Result<NonNull<Self>> {
        let body = voicevox_core::blocking::Synthesizer::builder(&onnxruntime.0)
            .text_analyzer(open_jtalk.body().clone())
            .acceleration_mode(acceleration_mode.into())
            .cpu_num_threads(cpu_num_threads)
            .build()?;
        Ok(<Self as CApiObject>::new(body))
    }
}

#[ext(VoicevoxSynthesizerPtrExt)]
impl *const VoicevoxSynthesizer {
    pub(crate) fn onnxruntime(self) -> &'static VoicevoxOnnxruntime {
        VoicevoxOnnxruntime::new(self.body().onnxruntime())
    }

    pub(crate) fn load_voice_model(
        self,
        model: &voicevox_core::blocking::VoiceModelFile,
        on_existing: voicevox_core::OnExistingVoiceModelId,
    ) -> CApiResult<()> {
        self.body()
            .load_voice_model(model)
            .on_existing(on_existing)
            .perform()?;
        Ok(())
    }

    pub(crate) fn unload_voice_model(self, model_id: VoiceModelId) -> Result<()> {
        self.body().unload_voice_model(model_id)?;
        Ok(())
    }

    pub(crate) fn metas(self) -> CString {
        metas_to_json(&self.body().metas())
    }
}

impl VoicevoxVoiceModelFile {
    pub(crate) fn open(path: impl AsRef<Path>) -> Result<NonNull<Self>> {
        let model = voicevox_core::blocking::VoiceModelFile::open(path)?;
        Ok(Self::new(model))
    }
}

#[ext(VoicevoxVoiceModelFilePtrExt)]
impl *const VoicevoxVoiceModelFile {
    pub(crate) fn metas(self) -> CString {
        metas_to_json(self.body().metas())
    }
}

#[ext(VoicevoxAudioFeaturePtrExt)]
impl *const VoicevoxAudioFeature {
    pub(crate) fn len(self) -> usize {
        self.body().len()
    }

    // easy-extの仕様上、`clippy::wrong_self_convention`が起動する
    // （しかも`#[expect]`が難しい位置で）ので`&self`
    pub(crate) fn is_empty(&self) -> bool {
        self.body().is_empty()
    }

    pub(crate) fn slice(
        self,
        start_inclusive: usize,
        end: usize,
        end_is_inclusive: bool,
        clamp: bool,
        allow_start_exceeds_end: bool,
    ) -> CApiResult<VoicevoxAudioFeatureSlice> {
        let range = AudioFeatureSlice::new(&self.body()).validate_range_(
            start_inclusive,
            end,
            end_is_inclusive,
            clamp,
            allow_start_exceeds_end,
        )?;
        let rust_api = self.body();
        let slice = rust_api.slice(range);
        Ok(VoicevoxAudioFeatureSlice {
            audio_feature: self,
            start_offset: slice.start_offset(),
            end: slice.start_offset() + slice.len() - usize::from(end_is_inclusive),
            end_is_inclusive,
            clamp: false,
            allow_start_exceeds_end: false,
        })
    }
}

impl VoicevoxAudioFeatureSlice {
    pub(crate) fn new(
        audio_feature: *const VoicevoxAudioFeature,
        clamp: bool,
        allow_start_exceeds_end: bool,
    ) -> Self {
        Self {
            audio_feature,
            start_offset: 0,
            end: audio_feature.len(),
            end_is_inclusive: false,
            clamp,
            allow_start_exceeds_end,
        }
    }

    pub(crate) fn validate(self) -> CApiResult<std::ops::Range<usize>> {
        AudioFeatureSlice::new(&self.audio_feature.body()).validate_range_(
            self.start_offset,
            self.end,
            self.end_is_inclusive,
            self.clamp,
            self.allow_start_exceeds_end,
        )
    }

    pub(crate) fn with_rust_api<F, O>(self, f: F) -> CApiResult<O>
    where
        F: FnOnce(AudioFeatureSlice<'_>) -> CApiResult<O>,
    {
        let range = self.validate()?;
        f(self.audio_feature.body().slice(range))
    }

    pub(crate) fn len(self) -> CApiResult<usize> {
        self.with_rust_api(|rust_api| Ok(rust_api.len()))
    }

    pub(crate) fn is_empty(self) -> CApiResult<bool> {
        self.with_rust_api(|rust_api| Ok(rust_api.is_empty()))
    }

    pub(crate) fn slice(
        self,
        start_inclusive: usize,
        end: usize,
        end_is_inclusive: bool,
        clamp: bool,
        allow_start_exceeds_end: bool,
    ) -> CApiResult<Self> {
        self.with_rust_api(|rust_api| {
            let range = rust_api.validate_range_(
                start_inclusive,
                end,
                end_is_inclusive,
                clamp,
                allow_start_exceeds_end,
            )?;
            let rust_api = rust_api.slice(range);
            Ok(Self {
                audio_feature: self.audio_feature,
                start_offset: rust_api.start_offset(),
                end: rust_api.start_offset() + rust_api.len() - usize::from(end_is_inclusive),
                end_is_inclusive,
                clamp: false,
                allow_start_exceeds_end: false,
            })
        })
    }
}

#[ext]
impl AudioFeatureSlice<'_> {
    fn validate_range_(
        self,
        start_inclusive: usize,
        end: usize,
        end_is_inclusive: bool,
        clamp: bool,
        allow_start_exceeds_end: bool,
    ) -> CApiResult<std::ops::Range<usize>> {
        let len = self.len();
        let result = if end_is_inclusive {
            let range = if clamp {
                let last = len
                    .checked_sub(1)
                    .ok_or(CApiError::EndOutOfBounds { end, len })?;
                cmp::min(start_inclusive, last)..=cmp::min(end, last)
            } else {
                start_inclusive..=end
            };
            self.validate_range(range)
        } else {
            let range = if clamp {
                cmp::min(start_inclusive, len)..cmp::min(end, len)
            } else {
                start_inclusive..end
            };
            self.validate_range(range)
        };

        match result {
            Ok(range) => Ok(range),
            Err(AudioFeatureOutOfBoundsError::StartOutOfBounds { start, len }) => {
                Err(CApiError::StartOutOfBounds { start, len })
            }
            Err(AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len }) => {
                Err(CApiError::EndOutOfBounds { end, len })
            }
            Err(AudioFeatureOutOfBoundsError::StartExceedsEnd { start, end }) => {
                if allow_start_exceeds_end {
                    Ok(start..start)
                } else {
                    Err(CApiError::StartExceedsEnd { start, end })
                }
            }
            Err(AudioFeatureOutOfBoundsError::Unknown { .. }) => unreachable!(
                "`validate_range` should not return `Unknown` for `Range` and `RangeInclusive`"
            ),
        }
    }
}

fn metas_to_json(metas: &[CharacterMeta]) -> CString {
    let metas = serde_json::to_string(metas).expect("should not fail");
    CString::new(metas).expect("should not contain NUL")
}

#[duplicate_item(
    H                          B;
    [ OpenJtalkRc ]            [ voicevox_core::blocking::OpenJtalk ];
    [ VoicevoxUserDict ]       [ voicevox_core::blocking::UserDict ];
    [ VoicevoxSynthesizer ]    [ voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk> ];
    [ VoicevoxVoiceModelFile ] [ voicevox_core::blocking::VoiceModelFile ];
    [ VoicevoxAudioFeature ]   [ voicevox_core::AudioFeature ];
)]
impl CApiObject for H {
    type RustApiObject = B;

    fn known_addrs() -> &'static std::sync::Mutex<HashSet<NonZero<usize>>> {
        static KNOWN_ADDRS: LazyLock<std::sync::Mutex<HashSet<NonZero<usize>>>> =
            LazyLock::new(Default::default);
        &KNOWN_ADDRS
    }

    fn heads() -> &'static boxcar::Vec<Self> {
        static HEADS: boxcar::Vec<H> = boxcar::Vec::new();
        &HEADS
    }

    fn bodies() -> &'static std::sync::Mutex<
        HashMap<NonZero<usize>, Arc<parking_lot::RwLock<MaybeDeleted<Self::RustApiObject>>>>,
    > {
        #[expect(clippy::type_complexity, reason = "`CApiObject::bodies`と同様")]
        static BODIES: LazyLock<
            std::sync::Mutex<HashMap<NonZero<usize>, Arc<parking_lot::RwLock<MaybeDeleted<B>>>>>,
        > = LazyLock::new(Default::default);

        &BODIES
    }
}
