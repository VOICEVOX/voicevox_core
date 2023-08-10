use std::{
    ffi::{CStr, CString},
    path::Path,
    sync::Arc,
};

use voicevox_core::{InitializeOptions, OpenJtalk, Result, Synthesizer, VoiceModel, VoiceModelId};

use crate::{OpenJtalkRc, VoicevoxSynthesizer, VoicevoxVoiceModel};

impl OpenJtalkRc {
    pub(crate) fn new_with_initialize(open_jtalk_dic_dir: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            open_jtalk: Arc::new(OpenJtalk::new_with_initialize(open_jtalk_dic_dir)?),
        })
    }
}

impl VoicevoxSynthesizer {
    pub(crate) async fn new_with_initialize(
        open_jtalk: &OpenJtalkRc,
        options: &InitializeOptions,
    ) -> Result<Self> {
        Ok(Self {
            synthesizer: Synthesizer::new_with_initialize(open_jtalk.open_jtalk.clone(), options)
                .await?,
            metas_cstring: CString::default(),
        })
    }

    pub(crate) async fn load_voice_model(&mut self, model: &VoiceModel) -> Result<()> {
        self.synthesizer.load_voice_model(model).await?;
        let metas = self.synthesizer.metas();
        self.metas_cstring = CString::new(serde_json::to_string(metas).unwrap()).unwrap();
        Ok(())
    }

    pub(crate) fn unload_voice_model(&mut self, model_id: &VoiceModelId) -> Result<()> {
        self.synthesizer.unload_voice_model(model_id)?;
        let metas = self.synthesizer.metas();
        self.metas_cstring = CString::new(serde_json::to_string(metas).unwrap()).unwrap();
        Ok(())
    }

    pub(crate) fn metas(&self) -> &CStr {
        &self.metas_cstring
    }
}

impl VoicevoxVoiceModel {
    pub(crate) async fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let model = VoiceModel::from_path(path).await?;
        let id = CString::new(model.id().raw_voice_model_id().as_str()).unwrap();
        let metas = CString::new(serde_json::to_string(model.metas()).unwrap()).unwrap();
        Ok(Self { model, id, metas })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::test_util::OPEN_JTALK_DIC_DIR;
    use rstest::*;

    #[rstest]
    #[tokio::test]
    async fn new_with_initialize_must_output_metas_json() {
        let open_jtalk = OpenJtalkRc::new_with_initialize(OPEN_JTALK_DIC_DIR).unwrap();
        let mut options = InitializeOptions::default();
        options.load_all_models = true;
        let synthesizer = VoicevoxSynthesizer::new_with_initialize(&open_jtalk, &options)
            .await
            .unwrap();

        println!("{:?}", synthesizer.metas());
        assert_eq!(CStr::is_empty(synthesizer.metas()), false);
    }
}
