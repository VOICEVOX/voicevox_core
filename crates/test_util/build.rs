use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::ensure;
use async_std::io::ReadExt as _;
use flate2::read::GzDecoder;
use tar::Archive;

const DIC_DIR_NAME: &str = "open_jtalk_dic_utf_8-1.11";

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    download_open_jtalk_dict(out_dir).await
}

async fn download_open_jtalk_dict(out_dir: &Path) -> anyhow::Result<()> {
    let download_url = format!(
        "https://github.com/r9y9/open_jtalk/releases/download/v1.11.1/{DIC_DIR_NAME}.tar.gz"
    );

    let req = surf::get(download_url);
    let client = surf::client().with(surf::middleware::Redirect::default());
    let mut res = client.send(req).await.map_err(surf::Error::into_inner)?;
    ensure!(res.status() == 200, "{}", res.status());
    let mut body_bytes = Vec::with_capacity(100 * 1024 * 1024);
    res.read_to_end(&mut body_bytes).await?;
    let dict_tar = GzDecoder::new(&body_bytes[..]);

    let mut dict_archive = Archive::new(dict_tar);
    dict_archive.unpack(out_dir)?;
    Ok(())
}
