use std::str;

use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Deserializer};

pub(crate) static SNAPSHOTS: Lazy<Snapshots> =
    Lazy::new(|| toml::from_str(include_str!("./snapshots.toml")).unwrap());

#[derive(Deserialize)]
pub(crate) struct Snapshots {
    pub(crate) compatible_engine: CompatibleEngine,
    pub(crate) compatible_engine_load_model_before_initialize:
        CompatibleEngineLoadModelBeforeInitialize,
}

#[derive(Deserialize)]
pub(crate) struct CompatibleEngine {
    pub(crate) yukarin_s_forward: [f32; 8],
    pub(crate) yukarin_sa_forward: [f32; 5],
    #[serde(deserialize_with = "deserialize_platform_specific_snapshot")]
    pub(crate) stderr: String,
}

#[derive(Deserialize)]
pub(crate) struct CompatibleEngineLoadModelBeforeInitialize {
    pub(crate) last_error_message: String,
    #[serde(deserialize_with = "deserialize_platform_specific_snapshot")]
    pub(crate) stderr: String,
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
