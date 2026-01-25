use std::{env, sync::LazyLock};

#[cfg(not(any(feature = "load-onnxruntime", feature = "link-onnxruntime")))]
compile_error!("either `load-onnxruntime` or `link-onnxruntime` must be enabled");

const ENV_FORCE_DOWNLOAD_ORT: &str = "VVCORE_BUILD_FORCE_DOWNLOAD_ORT";
const ENV_FORCE_COPY_DOWNLOADED_ORT: &str = "VVCORE_BUILD_FORCE_COPY_DOWNLOADED_ORT";

static FORCE_DOWNLOAD_ORT: LazyLock<bool> = LazyLock::new(|| is_true(ENV_FORCE_DOWNLOAD_ORT));
static FORCE_COPY_DOWNLOADED_ORT: LazyLock<bool> =
    LazyLock::new(|| is_true(ENV_FORCE_COPY_DOWNLOADED_ORT));

fn main() -> Result<(), build_features::Error> {
    if env::var("DOCS_RS").is_ok() {
        return Ok(());
    }

    println!("cargo::rerun-if-env-changed={ENV_FORCE_DOWNLOAD_ORT}");
    println!("cargo::rerun-if-env-changed={ENV_FORCE_COPY_DOWNLOADED_ORT}");

    println!("cargo::rerun-if-changed=build.rs");

    #[cfg(feature = "link-onnxruntime")]
    {
        build_features::link::link(!*FORCE_DOWNLOAD_ORT)?;
    }

    if cfg!(feature = "link-onnxruntime") || *FORCE_DOWNLOAD_ORT {
        build_features::download::download(
            cfg!(feature = "link-onnxruntime"),
            cfg!(feature = "link-onnxruntime") || *FORCE_COPY_DOWNLOADED_ORT,
        )?;
    }

    Ok(())
}

fn is_true(env_name: &'static str) -> bool {
    env::var(env_name).is_ok_and(|s| ["1", "true"].contains(&&*s.to_ascii_lowercase()))
}
