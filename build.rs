extern crate cbindgen;

use cbindgen::{Config, EnumConfig, FunctionConfig, Language, RenameRule};
use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = target_dir()
        .join(format!("{}.h", package_name))
        .display()
        .to_string();

    let config = Config {
        language: Language::C,
        sys_includes: vec!["stdint.h".into()],
        no_includes: true,
        cpp_compat: true,
        include_guard: Some("VOICEVOX_CORE_INCLUDE_GUARD".into()),
        function: FunctionConfig {
            prefix: Some(
                r#"#ifdef _WIN32
__declspec(dllimport)
#endif
"#
                .into(),
            ),
            ..Default::default()
        },
        enumeration: EnumConfig {
            rename_variants: RenameRule::ScreamingSnakeCase,
            ..Default::default()
        },
        ..Default::default()
    };

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&output_file);
}
fn target_dir() -> PathBuf {
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target)
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target")
    }
}
