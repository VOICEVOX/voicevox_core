fn main() {
    #[cfg(feature = "generate-c-header")]
    generate_c_header();

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/");
        println!("cargo:rustc-link-arg=-Wl,-install_name,@rpath/libvoicevox_core.dylib");
    }
}

#[cfg(feature = "generate-c-header")]
fn generate_c_header() {
    use std::env;
    use std::path::PathBuf;

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = target_dir().join("voicevox_core.h").display().to_string();

    cbindgen::generate(&crate_dir)
        .unwrap()
        .write_to_file(&output_file);

    fn target_dir() -> PathBuf {
        PathBuf::from(env::var("CARGO_WORKSPACE_DIR").unwrap()).join("target")
    }
}
