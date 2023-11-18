use std::{path::Path, sync::Arc};

use crate::{AccelerationMode, InitializeOptions, OpenJtalk, Synthesizer, VoiceModel};

pub async fn synthesizer_with_sample_voice_model(
    open_jtalk_dic_dir: impl AsRef<Path>,
) -> anyhow::Result<Synthesizer<Arc<OpenJtalk>>> {
    let syntesizer = Synthesizer::new(
        Arc::new(OpenJtalk::new(open_jtalk_dic_dir).unwrap()),
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
