use std::{collections::BTreeMap, fmt::Display};

use derive_getters::Getters;
use derive_new::new;
use serde::{Deserialize, Serialize};

use super::*;

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

/// モデル内IDの実体
pub type RawModelInnerId = u32;
/// モデル内ID
#[derive(PartialEq, Eq, Clone, Copy, Ord, PartialOrd, Deserialize, Serialize, new, Debug)]
pub struct ModelInnerId(RawModelInnerId);

impl ModelInnerId {
    pub fn raw_id(self) -> RawModelInnerId {
        self.0
    }
}

impl Display for ModelInnerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_id())
    }
}

#[derive(Deserialize, Getters, Clone)]
pub struct Manifest {
    manifest_version: ManifestVersion,
    metas_filename: String,
    decode_filename: String,
    predict_duration_filename: String,
    predict_intonation_filename: String,
    #[serde(default)]
    style_id_to_model_inner_id: BTreeMap<StyleId, ModelInnerId>,
}
