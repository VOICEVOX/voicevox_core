extern crate cbindgen;

use cbindgen::{Config, EnumConfig, FunctionConfig, Language, RenameRule};
use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = target_dir().join("core.h").display().to_string();
    let config = Config {
        language: Language::C,
        no_includes: true,
        after_includes: Some(
            r#"#ifdef __cplusplus
#include <cstdint>
#else // __cplusplus
#include <stdbool.h>
#include <stdint.h>
#endif // __cplusplus"#
                .into(),
        ),
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
    PathBuf::from(env::var("CARGO_WORKSPACE_DIR").unwrap()).join("target")
}
