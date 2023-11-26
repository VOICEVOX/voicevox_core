use std::path::Path;

use crate::{AccelerationMode, InitializeOptions, OpenJtalk, Synthesizer, VoiceModel};

pub async fn synthesizer_with_sample_voice_model(
    open_jtalk_dic_dir: impl AsRef<Path>,
) -> anyhow::Result<Synthesizer<OpenJtalk>> {
    let syntesizer = Synthesizer::new(
        OpenJtalk::new(open_jtalk_dic_dir).await?,
        &InitializeOptions {
            acceleration_mode: AccelerationMode::Cpu,
            ..Default::default()
        },
    )?;

    let model = &VoiceModel::from_path(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../model/sample.vvm",
    ))
    .await?;
    syntesizer.load_voice_model(model).await?;

    Ok(syntesizer)
}
