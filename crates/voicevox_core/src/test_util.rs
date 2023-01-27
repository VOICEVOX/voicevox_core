use std::path::{Path, PathBuf};

use async_std::{fs, io::ReadExt, sync::Mutex};
use flate2::read::GzDecoder;
use once_cell::sync::Lazy;
use tar::Archive;

const DIC_DIR_NAME: &str = "open_jtalk_dic_utf_8-1.11";
static OPEN_JTALK_DIC_DIR: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

pub async fn download_open_jtalk_dict_if_no_exists() -> PathBuf {
    let mut open_jtalk_dic_dir = OPEN_JTALK_DIC_DIR.lock().await;
    if open_jtalk_dic_dir.is_none() {
        let dic_dir = PathBuf::from(env!("OUT_DIR"))
            .join("testdata/open_jtalk_dic")
            .join(DIC_DIR_NAME);
        if !dic_dir.exists() {
            fs::create_dir_all(&dic_dir).await.unwrap();
            download_open_jtalk_dict(&dic_dir).await;
        }
        *open_jtalk_dic_dir = Some(dic_dir);
    }
    PathBuf::from(open_jtalk_dic_dir.as_ref().unwrap())
}

async fn download_open_jtalk_dict(open_jtalk_dic_dir: impl AsRef<Path>) {
    let download_url = format!(
        "https://github.com/r9y9/open_jtalk/releases/download/v1.11.1/{DIC_DIR_NAME}.tar.gz"
    );

    let req = surf::get(download_url);
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
