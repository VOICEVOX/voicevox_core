use std::{collections::BTreeMap, fmt::Display, sync::Arc};

use derive_getters::Getters;
use derive_more::Deref;
use derive_new::new;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::StyleId;

pub type RawManifestVersion = String;
#[derive(Deserialize, Clone, Debug, PartialEq, new)]
pub struct ManifestVersion(RawManifestVersion);

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
    // FIXME: UUIDにする
    // https://github.com/VOICEVOX/voicevox_core/issues/581
    #[allow(dead_code)]
    manifest_version: ManifestVersion,
    metas_filename: String,
    #[serde(flatten)]
    domains: ManifestDomains,
}

#[derive(Deserialize, Clone)]
pub(crate) struct ManifestDomains {
    pub(crate) talk: Option<TalkManifest>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct TalkManifest {
    pub(crate) predict_duration_filename: String,
    pub(crate) predict_intonation_filename: String,
    pub(crate) decode_filename: String,
    #[serde(default)]
    pub(crate) style_id_to_model_inner_id: StyleIdToModelInnerId,
}

#[serde_as]
#[derive(Default, Clone, Deref, Deserialize)]
#[deref(forward)]
pub(crate) struct StyleIdToModelInnerId(
    #[serde_as(as = "Arc<BTreeMap<DisplayFromStr, _>>")] Arc<BTreeMap<StyleId, ModelInnerId>>,
);
