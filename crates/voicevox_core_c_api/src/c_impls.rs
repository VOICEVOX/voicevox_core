use std::{ffi::CString, path::Path};

use camino::Utf8Path;
use ref_cast::ref_cast_custom;
use voicevox_core::{InitializeOptions, Result, SpeakerMeta, VoiceModelId};

use crate::{
    helpers::CApiResult, OpenJtalkRc, VoicevoxOnnxruntime, VoicevoxSynthesizer,
    VoicevoxVoiceModelFile,
};

// FIXME: 中身(Rust API)を直接操作するかラッパーメソッド越しにするのかが混在していて、一貫性を
// 欠いている

impl VoicevoxOnnxruntime {
    #[cfg(feature = "load-onnxruntime")]
    pub(crate) fn lib_versioned_filename() -> &'static std::ffi::CStr {
        to_cstr!(voicevox_core::blocking::Onnxruntime::LIB_VERSIONED_FILENAME)
    }

    #[cfg(feature = "load-onnxruntime")]
    pub(crate) fn lib_unversioned_filename() -> &'static std::ffi::CStr {
        to_cstr!(voicevox_core::blocking::Onnxruntime::LIB_UNVERSIONED_FILENAME)
    }

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
            .exec()?;
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
        const __RUST_STR: &str = $s;
        static __C_STR: &[u8] = const_format::concatcp!(__RUST_STR, '\0').as_bytes();

        std::ffi::CStr::from_bytes_with_nul(__C_STR)
            .unwrap_or_else(|e| panic!("{__RUST_STR:?} should not contain `\\0`: {e}"))
    }};
}
#[cfg(feature = "load-onnxruntime")]
use to_cstr;

impl OpenJtalkRc {
    pub(crate) fn new(open_jtalk_dic_dir: impl AsRef<Utf8Path>) -> Result<Self> {
        Ok(Self {
            open_jtalk: voicevox_core::blocking::OpenJtalk::new(open_jtalk_dic_dir)?,
        })
    }
}

impl VoicevoxSynthesizer {
    pub(crate) fn new(
        onnxruntime: &'static VoicevoxOnnxruntime,
        open_jtalk: &OpenJtalkRc,
        options: &InitializeOptions,
    ) -> Result<Self> {
        let synthesizer = voicevox_core::blocking::Synthesizer::new(
            &onnxruntime.0,
            open_jtalk.open_jtalk.clone(),
            options,
        )?;
        Ok(Self { synthesizer })
    }

    pub(crate) fn onnxruntime(&self) -> &'static VoicevoxOnnxruntime {
        VoicevoxOnnxruntime::new(self.synthesizer.onnxruntime())
    }

    pub(crate) fn load_voice_model(
        &self,
        model: &voicevox_core::blocking::VoiceModelFile,
    ) -> CApiResult<()> {
        self.synthesizer.load_voice_model(model)?;
        Ok(())
    }

    pub(crate) fn unload_voice_model(&self, model_id: VoiceModelId) -> Result<()> {
        self.synthesizer.unload_voice_model(model_id)?;
        Ok(())
    }

    pub(crate) fn metas(&self) -> CString {
        metas_to_json(&self.synthesizer.metas())
    }
}

impl VoicevoxVoiceModelFile {
    pub(crate) fn open(path: impl AsRef<Path>) -> Result<Self> {
        let model = voicevox_core::blocking::VoiceModelFile::open(path)?;
        Ok(Self { model })
    }

    pub(crate) fn metas(&self) -> CString {
        metas_to_json(self.model.metas())
    }
}

fn metas_to_json(metas: &[SpeakerMeta]) -> CString {
    let metas = serde_json::to_string(metas).expect("should not fail");
    CString::new(metas).expect("should not contain NUL")
}
