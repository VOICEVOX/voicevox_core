use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::Mutex,
};

use async_std::{fs, io::ReadExt};
use flate2::read::GzDecoder;
use once_cell::sync::Lazy;
use tar::Archive;

const DIC_DIR_NAME: &str = "open_jtalk_dic_utf_8-1.11";
static OPEN_JTALK_DIC_DIR: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

#[allow(dead_code)]
pub async fn create_user_dict_if_not_exists_for_test(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let mut hasher = DefaultHasher::new();
    path.to_str().unwrap().hash(&mut hasher);
    let out_path = PathBuf::from(env!("OUT_DIR"))
        .join("testdata")
        .join(hasher.finish().to_string());
    if !out_path.exists() {
        create_user_dict(path, &out_path).await;
    }
    out_path
}

async fn create_user_dict(path: impl AsRef<Path>, out_path: impl AsRef<Path>) {
    let open_jtalk_dic_dir = download_open_jtalk_dict_if_no_exists().await;
    let out_path = out_path.as_ref();
    if !out_path.exists() {
        fs::create_dir_all(out_path.parent().unwrap())
            .await
            .unwrap();

        open_jtalk::mecab_dict_index(&[
            "mecab-dict-index",
            "-d",
            open_jtalk_dic_dir.to_str().unwrap(),
            "-u",
            out_path.to_str().unwrap(),
            "-f",
            "utf-8",
            "-t",
            "utf-8",
            path.as_ref().to_str().unwrap(),
        ]);
    }
}

pub async fn download_open_jtalk_dict_if_no_exists() -> PathBuf {
    let mut open_jtalk_dic_dir = OPEN_JTALK_DIC_DIR.lock().unwrap();
    if open_jtalk_dic_dir.is_none() {
        let dic_dir = PathBuf::from(env!("OUT_DIR"))
            .join("testdata/open_jtalk_dic")
            .join(DIC_DIR_NAME);
        if !dic_dir.exists() {
            fs::create_dir_all(&dic_dir).await.unwrap();
            downlaod_open_jtalk_dict(&dic_dir).await;
        }
        *open_jtalk_dic_dir = Some(dic_dir);
    }
    PathBuf::from(open_jtalk_dic_dir.as_ref().unwrap())
}

async fn downlaod_open_jtalk_dict(open_jtalk_dic_dir: impl AsRef<Path>) {
    let downlaod_url = format!(
        "https://github.com/r9y9/open_jtalk/releases/download/v1.11.1/{}.tar.gz",
        DIC_DIR_NAME
    );

    let req = surf::get(downlaod_url);
    let client = surf::client().with(surf::middleware::Redirect::default());
    let mut res = client.send(req).await.unwrap();
    let mut body_bytes = Vec::with_capacity(100 * 1024 * 1024);
    res.read_to_end(&mut body_bytes).await.unwrap();
    let dict_tar = GzDecoder::new(&body_bytes[..]);

    let mut dict_archive = Archive::new(dict_tar);
    let open_jtalk_dic_dir = open_jtalk_dic_dir.as_ref();
    dict_archive
        .unpack(open_jtalk_dic_dir.parent().unwrap())
        .unwrap();
}
