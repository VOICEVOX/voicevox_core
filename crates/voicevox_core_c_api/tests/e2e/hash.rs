use std::fmt::{self, Debug};

use serde::{de::Error as _, Deserialize, Deserializer};
use sha2::{Digest as _, Sha256};

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
