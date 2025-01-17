use std::{ffi::OsString, path::Path};

use camino::Utf8Path;

use crate::AccelerationMode;

pub use crate::synthesizer::nonblocking::IntoBlocking;

pub async fn synthesizer_with_sample_voice_model(
    voice_model_path: impl AsRef<Path>,
    #[cfg_attr(feature = "link-onnxruntime", allow(unused_variables))] onnxruntime_dylib_path: impl Into<
        OsString,
    >,
    open_jtalk_dic_dir: impl AsRef<Utf8Path>,
) -> anyhow::Result<crate::nonblocking::Synthesizer<crate::nonblocking::OpenJtalk>> {
    let syntesizer = crate::nonblocking::Synthesizer::builder(
        #[cfg(feature = "load-onnxruntime")]
        crate::nonblocking::Onnxruntime::load_once()
            .filename(onnxruntime_dylib_path)
            .perform()
            .await?,
        #[cfg(feature = "link-onnxruntime")]
        crate::nonblocking::Onnxruntime::init_once().await?,
    )
    .open_jtalk(crate::nonblocking::OpenJtalk::new(open_jtalk_dic_dir).await?)
    .acceleration_mode(AccelerationMode::Cpu)
    .build()?;

    let model = &crate::nonblocking::VoiceModelFile::open(voice_model_path).await?;
    syntesizer.load_voice_model(model).await?;

    Ok(syntesizer)
}
