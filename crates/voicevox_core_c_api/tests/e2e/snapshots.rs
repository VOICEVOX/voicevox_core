use std::{
    fmt::{self, Debug},
    str,
};

use once_cell::sync::Lazy;
use serde::{
    de::{DeserializeOwned, Error as _},
    Deserialize, Deserializer,
};
use sha2::{Digest as _, Sha256};

pub(crate) static SNAPSHOTS: Lazy<Snapshots> =
    Lazy::new(|| toml::from_str(include_str!("./snapshots.toml")).unwrap());

#[derive(Deserialize)]
pub(crate) struct Snapshots {
    pub(crate) compatible_engine: CompatibleEngine,
}

#[derive(Deserialize)]
pub(crate) struct CompatibleEngine {
    pub(crate) yukarin_s_forward: GhActionsWindows<Sha256Sum>,
    pub(crate) yukarin_sa_forward: GhActionsWindows<Sha256Sum>,
    pub(crate) decode_forward: GhActionsWindows<Sha256Sum>,
    #[serde(deserialize_with = "deserialize_platform_specific_snapshot")]
    pub(crate) stderr: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GhActionsWindows<T> {
    pub(crate) gh_actions_windows: T,
}

#[derive(PartialEq)]
pub(crate) struct Sha256Sum([u8; 32]);

impl Sha256Sum {
    pub(crate) fn new(data: impl AsRef<[u8]>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        Self(hasher.finalize().into())
    }

    pub(crate) fn le_bytes(values: &[f32]) -> Self {
        let bytes = values
            .iter()
            .copied()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        Self::new(bytes)
    }
}

impl<'de> Deserialize<'de> for Sha256Sum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut sha256sum = [0; 32];
        hex::decode_to_slice(s, &mut sha256sum).map_err(D::Error::custom)?;
        Ok(Self(sha256sum))
    }
}

impl Debug for Sha256Sum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(&self.0[..]))
    }
}

fn deserialize_platform_specific_snapshot<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    let PlatformSpecificSnapshot { __value } = PlatformSpecificSnapshot::deserialize(deserializer)?;
    return Ok(__value);

    #[derive(Deserialize)]
    struct PlatformSpecificSnapshot<T> {
        #[cfg_attr(windows, serde(rename = "windows"))]
        #[cfg_attr(unix, serde(rename = "unix"))]
        __value: T,
    }
}
