use std::{ffi::CString, path::Path};

use voicevox_core::{InitializeOptions, Result, VoiceModelId};

use crate::{helpers::CApiResult, OpenJtalkRc, VoicevoxSynthesizer, VoicevoxVoiceModel};

impl OpenJtalkRc {
    pub(crate) fn new(open_jtalk_dic_dir: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            open_jtalk: voicevox_core::blocking::OpenJtalk::new(open_jtalk_dic_dir)?,
        })
    }
}

impl VoicevoxSynthesizer {
    pub(crate) fn new(open_jtalk: &OpenJtalkRc, options: &InitializeOptions) -> Result<Self> {
        let synthesizer =
            voicevox_core::blocking::Synthesizer::new(open_jtalk.open_jtalk.clone(), options)?;
        Ok(Self { synthesizer })
    }

    pub(crate) fn load_voice_model(
        &self,
        model: &voicevox_core::blocking::VoiceModel,
    ) -> CApiResult<()> {
        self.synthesizer.load_voice_model(model)?;
        Ok(())
    }

    pub(crate) fn unload_voice_model(&self, model_id: &VoiceModelId) -> Result<()> {
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
        let id = CString::new(model.id().raw_voice_model_id().as_str()).unwrap();
        let metas = CString::new(serde_json::to_string(model.metas()).unwrap()).unwrap();
        Ok(Self { model, id, metas })
    }
}
