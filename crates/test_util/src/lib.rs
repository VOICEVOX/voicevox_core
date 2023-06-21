mod typing;
pub use typing::*;

pub const OPEN_JTALK_DIC_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/open_jtalk_dic_utf_8-1.11"
);

pub const TESTDATA_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/testdata.json"));

impl TestData {
    pub fn load() -> TestData {
        serde_json::from_str(TESTDATA_JSON).unwrap()
    }
}
