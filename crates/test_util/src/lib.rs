mod typing;

include!(concat!(env!("OUT_DIR"), "/sample_voice_model_file.rs"));

#[allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_extern_crates,
    clippy::missing_safety_doc,
    clippy::too_many_arguments
)]
pub mod c_api {
    include!(concat!(env!("OUT_DIR"), "/c_api.rs"));

    pub const SAMPLE_VOICE_MODEL_FILE_PATH: &std::ffi::CStr = super::SAMPLE_VOICE_MODEL_FILE_C_PATH;
    pub const VV_MODELS_ROOT_DIR: &str = super::VV_MODELS_ROOT_DIR;
}

use once_cell::sync::Lazy;

pub use self::typing::{
    DecodeExampleData, DurationExampleData, ExampleData, IntonationExampleData,
};

pub const OPEN_JTALK_DIC_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/open_jtalk_dic_utf_8-1.11"
);

const EXAMPLE_DATA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/example_data.json"
));

pub static EXAMPLE_DATA: Lazy<ExampleData> = Lazy::new(|| {
    serde_json::from_str(EXAMPLE_DATA_JSON).expect("failed to parse example_data.json")
});
