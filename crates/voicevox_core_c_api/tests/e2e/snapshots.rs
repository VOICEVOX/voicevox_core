use std::str;

use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Deserializer};

use crate::hash::Sha256Sum;

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
