use std::fmt::Display;

use derive_getters::Getters;
use derive_new::new;
use serde::Deserialize;

pub type RawManifestVersion = String;
#[derive(Deserialize, Clone, Debug, PartialEq, new)]
pub struct ManifestVersion(RawManifestVersion);

impl ManifestVersion {
    pub fn raw_manifest_version(&self) -> &RawManifestVersion {
        &self.0
    }
}

impl Display for ManifestVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize, Getters, Clone)]
pub struct Manifest {
    manifest_version: ManifestVersion,
    metas_filename: String,
    decode_filename: String,
    predict_duration_filename: String,
    predict_intonation_filename: String,
}
