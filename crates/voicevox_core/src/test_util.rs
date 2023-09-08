use std::path::PathBuf;

use crate::{Result, VoiceModel};

pub async fn open_default_vvm_file() -> VoiceModel {
    VoiceModel::from_path(
        ::test_util::convert_zip_vvm(
            PathBuf::from(env!("CARGO_WORKSPACE_DIR"))
                .join(file!())
                .parent()
                .unwrap()
                .join("test_data/model_sources")
                .join("load_model_works1"),
        )
        .await,
    )
    .await
    .unwrap()
}

impl VoiceModel {
    pub(crate) async fn sample() -> Result<Self> {
        return Self::from_path(PATH).await;

        static PATH: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "/model/sample.vvm");
    }
}
