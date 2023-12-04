use std::path::Path;

use crate::{AccelerationMode, InitializeOptions};

pub async fn synthesizer_with_sample_voice_model(
    open_jtalk_dic_dir: impl AsRef<Path>,
) -> anyhow::Result<crate::tokio::Synthesizer<crate::tokio::OpenJtalk>> {
    let syntesizer = crate::tokio::Synthesizer::new(
        crate::tokio::OpenJtalk::new(open_jtalk_dic_dir).await?,
        &InitializeOptions {
            acceleration_mode: AccelerationMode::Cpu,
            ..Default::default()
        },
    )?;

    let model = &crate::tokio::VoiceModel::from_path(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../model/sample.vvm",
    ))
    .await?;
    syntesizer.load_voice_model(model).await?;

    Ok(syntesizer)
}
