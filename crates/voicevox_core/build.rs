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
            r#"#ifdef _WIN32
#ifdef VOICEVOX_CORE_EXPORTS
#define VOICEVOX_CORE_API __declspec(dllexport)
#else  // VOICEVOX_CORE_EXPORTS
#define VOICEVOX_CORE_API __declspec(dllimport)
#endif  // VOICEVOX_CORE_EXPORTS
#else   // _WIN32
#define VOICEVOX_CORE_API
#endif  // _WIN32

#ifdef __cplusplus
#include <cstdint>
#else
#include <stdbool.h>
#include <stdint.h>
#endif"#
                .into(),
        ),
        cpp_compat: true,
        include_guard: Some("VOICEVOX_CORE_INCLUDE_GUARD".into()),
        function: FunctionConfig {
            prefix: Some("VOICEVOX_CORE_API".into()),
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
