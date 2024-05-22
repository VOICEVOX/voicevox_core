use ::test_util::SAMPLE_VOICE_MODEL_FILE_PATH;

use crate::Result;

impl crate::tokio::VoiceModel {
    pub(crate) async fn sample() -> Result<Self> {
        Self::from_path(SAMPLE_VOICE_MODEL_FILE_PATH).await
    }
}
