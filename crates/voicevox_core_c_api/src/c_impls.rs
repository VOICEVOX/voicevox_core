use derive_getters::Getters;
use std::{
    ffi::{CStr, CString},
    path::Path,
    sync::Arc,
};

use voicevox_core::{
    InitializeOptions, OpenJtalk, Result, VoiceModel, VoiceModelId, VoiceSynthesizer,
};

pub(crate) struct COpenJtalkRc {
    open_jtalk: Arc<OpenJtalk>,
}

impl COpenJtalkRc {
    pub(crate) fn new_with_initialize(open_jtalk_dic_dir: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            open_jtalk: Arc::new(OpenJtalk::new_with_initialize(open_jtalk_dic_dir)?),
        })
    }
}

#[derive(Getters)]
pub(crate) struct CVoiceSynthesizer {
    synthesizer: VoiceSynthesizer,
    metas_cstring: CString,
}

impl CVoiceSynthesizer {
    pub(crate) async fn new_with_initialize(
        open_jtalk: &COpenJtalkRc,
        options: &InitializeOptions,
    ) -> Result<Self> {
        Ok(Self {
            synthesizer: VoiceSynthesizer::new_with_initialize(
                open_jtalk.open_jtalk.clone(),
                options,
            )
            .await?,
            metas_cstring: CString::default(),
        })
    }

    pub(crate) async fn load_model(&mut self, model: &VoiceModel) -> Result<()> {
        self.synthesizer.load_model(model).await?;
        let metas = self.synthesizer.metas();
        self.metas_cstring = CString::new(serde_json::to_string(metas).unwrap()).unwrap();
        Ok(())
    }

    pub(crate) fn unload_model(&mut self, model_id: &VoiceModelId) -> Result<()> {
        self.synthesizer.unload_model(model_id)?;
        let metas = self.synthesizer.metas();
        self.metas_cstring = CString::new(serde_json::to_string(metas).unwrap()).unwrap();
        Ok(())
    }

    pub(crate) fn metas(&self) -> &CStr {
        &self.metas_cstring
    }
}

#[derive(Getters)]
pub(crate) struct CVoiceModel {
    model: VoiceModel,
    id: CString,
    metas: CString,
}

impl CVoiceModel {
    pub(crate) async fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let model = VoiceModel::from_path(path).await?;
        let id = CString::new(model.id().raw_voice_model_id().as_str()).unwrap();
        let metas = CString::new(serde_json::to_string(model.metas()).unwrap()).unwrap();
        Ok(Self { model, id, metas })
    }
}
