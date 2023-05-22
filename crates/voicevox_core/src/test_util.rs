use async_zip::{write::ZipFileWriter, Compression, ZipEntryBuilder};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};

use crate::VoiceModel;

pub async fn open_default_vvm_file() -> VoiceModel {
    VoiceModel::from_path(
        convert_zip_vvm(
            PathBuf::from(env!("CARGO_WORKSPACE_DIR"))
                .join(file!())
                .parent()
                .unwrap()
                .join("test_data/model_sources")
                .join("load_model_works1"),
        )
        .await,
    )
    .await
    .unwrap()
}

static PATH_MUTEX: Lazy<Mutex<HashMap<PathBuf, Mutex<()>>>> =
    Lazy::new(|| Mutex::new(HashMap::default()));

async fn convert_zip_vvm(dir: impl AsRef<Path>) -> PathBuf {
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
        let mut out_file = File::create(&out_file_path).await.unwrap();
        let mut writer = ZipFileWriter::new(&mut out_file);

        for entry in dir.read_dir().unwrap().flatten() {
            let entry_builder = ZipEntryBuilder::new(
                entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                Compression::Deflate,
            );
            let mut entry_writer = writer.write_entry_stream(entry_builder).await.unwrap();
            let mut file = File::open(entry.path()).await.unwrap();
            let mut buf = Vec::with_capacity(entry.metadata().unwrap().len() as usize);
            file.read_to_end(&mut buf).await.unwrap();
            entry_writer.write_all(&buf).await.unwrap();
            entry_writer.close().await.unwrap();
        }
        writer.close().await.unwrap();
    }
    out_file_path
}
