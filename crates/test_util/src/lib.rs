mod typing;
use once_cell::sync::Lazy;
pub use typing::*;

pub const OPEN_JTALK_DIC_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/open_jtalk_dic_utf_8-1.11"
);

pub const EXAMPLE_DATA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/example_data.json"
));

pub static EXAMPLE_DATA: Lazy<ExampleData> = Lazy::new(|| {
    serde_json::from_str(EXAMPLE_DATA_JSON).expect("failed to parse example_data.json")
});
