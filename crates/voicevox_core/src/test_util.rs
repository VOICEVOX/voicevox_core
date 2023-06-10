use std::path::PathBuf;

use crate::VoiceModel;

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
