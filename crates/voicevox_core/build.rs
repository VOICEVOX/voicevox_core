use std::{env, sync::LazyLock};

#[cfg(not(any(feature = "load-onnxruntime", feature = "link-onnxruntime")))]
compile_error!("either `load-onnxruntime` or `link-onnxruntime` must be enabled");

const ENV_DOWNLOAD_AND_COPY_ORT: &str = "VVCORE_BUILD_DOWNLOAD_AND_COPY_ORT";
static DOWNLOAD_AND_COPY_ORT: LazyLock<bool> = LazyLock::new(|| is_true(ENV_DOWNLOAD_AND_COPY_ORT));

fn main() -> Result<(), build_features::Error> {
    if env::var("DOCS_RS").is_ok() {
        return Ok(());
    }

    println!("cargo::rerun-if-changed=build.rs");

    if cfg!(feature = "buildtime-download-onnxruntime") {
        println!("cargo::rerun-if-env-changed={ENV_DOWNLOAD_AND_COPY_ORT}");
    }

    if cfg!(feature = "buildtime-download-onnxruntime") && !*DOWNLOAD_AND_COPY_ORT {
        println!(
            "cargo::warning=`buildtime-download-onnxruntime` feature is no-op \
             when `${ENV_DOWNLOAD_AND_COPY_ORT}` is not enabled",
        );
    }
    if !cfg!(feature = "buildtime-download-onnxruntime") && *DOWNLOAD_AND_COPY_ORT {
        println!(
            "cargo::warning=`${ENV_DOWNLOAD_AND_COPY_ORT}` is no-op \
             when `buildtime-download-onnxruntime` feature is disabled",
        );
    }

    #[cfg(feature = "link-onnxruntime")]
    {
        static ATTEMPT_PKG_CONFIG: LazyLock<bool> = LazyLock::new(|| {
            !(cfg!(feature = "buildtime-download-onnxruntime") && *DOWNLOAD_AND_COPY_ORT)
        });
        build_features::link::link(*ATTEMPT_PKG_CONFIG)?;
    }

    #[cfg(feature = "buildtime-download-onnxruntime")]
    if *DOWNLOAD_AND_COPY_ORT {
        const ADD_LINK_SEARCH: bool = cfg!(feature = "link-onnxruntime");
        build_features::download::download(ADD_LINK_SEARCH)?;
    }

    Ok(())
}

fn is_true(env_name: &'static str) -> bool {
    env::var(env_name).is_ok_and(|s| ["1", "true"].contains(&&*s.to_ascii_lowercase()))
}
