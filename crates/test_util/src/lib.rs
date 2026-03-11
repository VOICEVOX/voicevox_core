mod typing;

include!(concat!(env!("OUT_DIR"), "/sample_voice_model_file.rs"));

#[allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unsafe_op_in_unsafe_fn, // https://github.com/rust-lang/rust-bindgen/issues/3147
    unused_extern_crates,
    clippy::missing_safety_doc,
    clippy::too_many_arguments,
    reason = "bindgenが生成するコードのため。`#[expect]`ではなく`#[allow]`なのは、bindgenが生成\
              するコードがOSにより変わるため"
)]
pub mod c_api {
    include!(concat!(env!("OUT_DIR"), "/c_api.rs"));

    pub const SAMPLE_VOICE_MODEL_FILE_PATH: &std::ffi::CStr = super::SAMPLE_VOICE_MODEL_FILE_C_PATH;
    pub const VV_MODELS_ROOT_DIR: &str = super::VV_MODELS_ROOT_DIR;
}

use std::sync::LazyLock;

pub use self::typing::{
    DecodeExampleData, DurationExampleData, ExampleData, IntonationExampleData,
};

pub const ONNXRUNTIME_DYLIB_PATH: &str = {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    macro_rules! version {
        () => {
            include_str!("../../../onnxruntime-version.txt")
        };
    }

    #[cfg(target_os = "windows")]
    macro_rules! lib_versioned_filename {
        () => {
            "onnxruntime.dll"
        };
    }

    #[cfg(target_os = "linux")]
    macro_rules! lib_versioned_filename {
        () => {
            concat!("libonnxruntime.so.", version!())
        };
    }

    #[cfg(target_os = "macos")]
    macro_rules! lib_versioned_filename {
        () => {
            concat!("libonnxruntime.", version!(), ".dylib")
        };
    }

    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/voicevox_core/downloads/onnxruntime/",
        lib_versioned_filename!(),
    )
};

pub const OPEN_JTALK_DIC_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/open_jtalk_dic_utf_8-1.11"
);

const EXAMPLE_DATA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/example_data.json"
));

pub static EXAMPLE_DATA: LazyLock<ExampleData> = LazyLock::new(|| {
    serde_json::from_str(EXAMPLE_DATA_JSON).expect("failed to parse example_data.json")
});
