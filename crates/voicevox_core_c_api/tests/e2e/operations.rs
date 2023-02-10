use std::{
    fmt::{self, Debug},
    process::Output,
    str,
};

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{de::Error as _, Deserialize, Deserializer};
use sha2::{Digest as _, Sha256};

pub(crate) mod compatible_engine;
pub(crate) mod voicevox_get_version;

static SNAPSHOTS: Lazy<Snapshots> =
    Lazy::new(|| toml::from_str(include_str!("./operations/snapshots.toml")).unwrap());

#[derive(Deserialize)]
struct Snapshots {
    compatible_engine: compatible_engine::Snapshots,
}

#[derive(PartialEq)]
struct Sha256Sum([u8; 32]);

impl Sha256Sum {
    fn new(data: impl AsRef<[u8]>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        Self(hasher.finalize().into())
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

fn mask_timestamps(output: Output) -> Output {
    let stderr = str::from_utf8(&output.stderr)
        .map(|stderr| {
            TIMESTAMPS
                .replace_all(stderr, "{timestamp}")
                .into_owned()
                .into()
        })
        .unwrap_or(output.stderr);

    return Output { stderr, ..output };

    static TIMESTAMPS: Lazy<Regex> = Lazy::new(|| {
        "(?m)^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}.[0-9]{6}Z"
            .parse()
            .unwrap()
    });
}
