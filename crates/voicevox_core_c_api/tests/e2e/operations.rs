use once_cell::sync::Lazy;
use serde::Deserialize;
use sha2::{Digest as _, Sha256};

pub(crate) mod metas;
pub(crate) mod voicevox_get_version;

fn sha256(data: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

static SNAPSHOTS: Lazy<Snapshots> =
    Lazy::new(|| toml::from_str(include_str!("./operations/snapshots.toml")).unwrap());

#[derive(Deserialize)]
struct Snapshots {
    metas: metas::Snapshots,
}
