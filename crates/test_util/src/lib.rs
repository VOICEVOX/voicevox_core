mod typing;
pub use typing::*;

pub const OPEN_JTALK_DIC_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/open_jtalk_dic_utf_8-1.11"
);

pub const EXAMPLE_DATA_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/example_data.json"));

impl ExampleData {
    pub fn load() -> ExampleData {
        serde_json::from_str(EXAMPLE_DATA_JSON).unwrap()
    }
}
