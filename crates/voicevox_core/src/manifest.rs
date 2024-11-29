use std::{
    collections::BTreeMap,
    fmt::{self, Display},
    sync::Arc,
};

use derive_getters::Getters;
use derive_more::Deref;
use derive_new::new;
use macros::IndexForFields;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    infer::domains::{inference_domain_map_values, InferenceDomainMap, TalkOperation},
    StyleId, VoiceModelId,
};

#[derive(Clone)]
struct FormatVersionV1;

impl<'de> Deserialize<'de> for FormatVersionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        return deserializer.deserialize_any(Visitor);

        struct Visitor;

        impl de::Visitor<'_> for Visitor {
            type Value = FormatVersionV1;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an unsigned integer")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    1 => Ok(FormatVersionV1),
                    v => Err(E::custom(format!(
                        "未知の形式です（`vvm_format_version={v}`）。新しいバージョンのVOICEVOX \
                         COREであれば対応しているかもしれません",
                    ))),
                }
            }
        }
    }
}

/// モデル内IDの実体
pub type RawInnerVoiceId = u32;
/// モデル内ID
#[derive(PartialEq, Eq, Clone, Copy, Ord, PartialOrd, Deserialize, Serialize, new, Debug)]
pub struct InnerVoiceId(RawInnerVoiceId);

impl InnerVoiceId {
    pub fn raw_id(self) -> RawInnerVoiceId {
        self.0
    }
}

impl Display for InnerVoiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_id())
    }
}

#[derive(Deserialize, Getters)]
pub struct Manifest {
    #[expect(dead_code, reason = "現状はバリデーションのためだけに存在")]
    vvm_format_version: FormatVersionV1,
    pub(crate) id: VoiceModelId,
    metas_filename: String,
    #[serde(flatten)]
    domains: InferenceDomainMap<ManifestDomains>,
}

pub(crate) type ManifestDomains = inference_domain_map_values!(for<D> Option<D::Manifest>);

#[derive(Deserialize, IndexForFields)]
#[cfg_attr(test, derive(Default))]
#[index_for_fields(TalkOperation)]
pub(crate) struct TalkManifest {
    #[index_for_fields(TalkOperation::PredictDuration)]
    pub(crate) predict_duration_filename: Arc<str>,

    #[index_for_fields(TalkOperation::PredictIntonation)]
    pub(crate) predict_intonation_filename: Arc<str>,

    #[index_for_fields(TalkOperation::GenerateFullIntermediate)]
    pub(crate) generate_full_intermediate_filename: Arc<str>,

    #[index_for_fields(TalkOperation::RenderAudioSegment)]
    pub(crate) render_audio_segment_filename: Arc<str>,

    #[serde(default)]
    pub(crate) style_id_to_inner_voice_id: StyleIdToInnerVoiceId,
}

#[serde_as]
#[derive(Default, Clone, Deref, Deserialize)]
#[deref(forward)]
pub(crate) struct StyleIdToInnerVoiceId(
    #[serde_as(as = "Arc<BTreeMap<DisplayFromStr, _>>")] Arc<BTreeMap<StyleId, InnerVoiceId>>,
);

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use rstest::rstest;
    use serde::Deserialize;

    use super::FormatVersionV1;

    #[rstest]
    #[case("{\"vvm_format_version\":1}", Ok(()))]
    #[case(
        "{\"vvm_format_version\":2}",
        Err(
            "未知の形式です（`vvm_format_version=2`）。新しいバージョンのVOICEVOX COREであれば対応\
             しているかもしれません at line 1 column 23",
        )
    )]
    fn vvm_format_version_works(
        #[case] input: &str,
        #[case] expected: Result<(), &str>,
    ) -> anyhow::Result<()> {
        let actual = serde_json::from_str::<ManifestPart>(input).map_err(|e| e.to_string());
        let actual = actual.as_ref().map(|_| ()).map_err(Deref::deref);
        assert_eq!(expected, actual);
        return Ok(());

        #[derive(Deserialize)]
        struct ManifestPart {
            #[expect(dead_code, reason = "バリデーションのためだけに存在")]
            vvm_format_version: FormatVersionV1,
        }
    }
}
