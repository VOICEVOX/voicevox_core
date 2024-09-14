use ::test_util::SAMPLE_VOICE_MODEL_FILE_PATH;

use crate::Result;

impl crate::nonblocking::VoiceModelFile {
    pub(crate) async fn sample() -> Result<Self> {
        Self::open(SAMPLE_VOICE_MODEL_FILE_PATH).await
    }
}
