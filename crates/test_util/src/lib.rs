mod typing;

use async_zip::{base::write::ZipFileWriter, Compression, ZipEntryBuilder};
use futures_lite::AsyncWriteExt as _;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
    sync::Mutex,
};

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

static PATH_MUTEX: Lazy<Mutex<HashMap<PathBuf, Mutex<()>>>> =
    Lazy::new(|| Mutex::new(HashMap::default()));

pub async fn convert_zip_vvm(dir: impl AsRef<Path>) -> PathBuf {
    let dir = dir.as_ref();
    let output_file_name = dir.file_name().unwrap().to_str().unwrap().to_owned() + ".vvm";

    let out_file_path = PathBuf::from(env!("OUT_DIR"))
        .join("test_data/models/")
        .join(output_file_name);
    let mut path_map = PATH_MUTEX.lock().await;
    if !path_map.contains_key(&out_file_path) {
        path_map.insert(out_file_path.clone(), Mutex::new(()));
    }
    let _m = path_map.get(&out_file_path).unwrap().lock().await;

    if !out_file_path.exists() {
        fs::create_dir_all(out_file_path.parent().unwrap())
            .await
            .unwrap();
        let mut writer = ZipFileWriter::new(vec![]);

        for entry in dir.read_dir().unwrap().flatten() {
            let entry_builder = ZipEntryBuilder::new(
                entry.path().file_name().unwrap().to_str().unwrap().into(),
                Compression::Deflate,
            );
            let mut entry_writer = writer.write_entry_stream(entry_builder).await.unwrap();
            let mut file = File::open(entry.path()).await.unwrap();
            let mut buf = Vec::with_capacity(entry.metadata().unwrap().len() as usize);
            file.read_to_end(&mut buf).await.unwrap();
            entry_writer.write_all(&buf).await.unwrap();
            entry_writer.close().await.unwrap();
        }
        let zip = writer.close().await.unwrap();
        fs::write(&out_file_path, zip).await.unwrap();
    }
    out_file_path
}
