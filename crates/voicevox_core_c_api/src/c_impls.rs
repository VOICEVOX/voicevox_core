use std::{ffi::CString, path::Path};

use camino::Utf8Path;
use ref_cast::ref_cast_custom;
use voicevox_core::{InitializeOptions, Result, VoiceModelId};

use crate::{
    helpers::CApiResult, OpenJtalkRc, VoicevoxOnnxruntime, VoicevoxSynthesizer, VoicevoxVoiceModel,
};

// FIXME: 中身(Rust API)を直接操作するかラッパーメソッド越しにするのかが混在していて、一貫性を
// 欠いている

impl VoicevoxOnnxruntime {
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
        model: &voicevox_core::blocking::VoiceModel,
    ) -> CApiResult<()> {
        self.synthesizer.load_voice_model(model)?;
        Ok(())
    }

    pub(crate) fn unload_voice_model(&self, model_id: VoiceModelId) -> Result<()> {
        self.synthesizer.unload_voice_model(model_id)?;
        Ok(())
    }

    pub(crate) fn metas(&self) -> CString {
        let metas = &self.synthesizer.metas();
        CString::new(serde_json::to_string(metas).unwrap()).unwrap()
    }
}

impl VoicevoxVoiceModel {
    pub(crate) fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let model = voicevox_core::blocking::VoiceModel::from_path(path)?;
        let metas = CString::new(serde_json::to_string(model.metas()).unwrap()).unwrap();
        Ok(Self { model, metas })
    }
}
