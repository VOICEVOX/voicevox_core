//! `vvm_format_version=1`での`Manifest`と`CharacterMeta`。

use serde::Deserialize;

use crate::{
    CharacterMeta, CharacterVersion, StyleId, StyleMeta, StyleType, VoiceModelId,
    core::{
        infer::domains::{FrameDecodeDomain, InferenceDomainMap, SingingTeacherDomain, TalkDomain},
        manifest::{ManifestBody, ManifestDomain},
    },
};

#[derive(Deserialize)]
pub(in super::super::super) struct ManifestBodyV1 {
    id: VoiceModelId,
    metas_filename: String,
    talk: Option<ManifestDomain<TalkDomain>>,
    singing_teacher: Option<ManifestDomain<SingingTeacherDomain>>,
    frame_decode: Option<ManifestDomain<FrameDecodeDomain>>,
}

impl From<ManifestBodyV1> for ManifestBody {
    fn from(v1: ManifestBodyV1) -> Self {
        Self {
            id: v1.id,
            metas_filename: v1.metas_filename,
            domains: InferenceDomainMap {
                talk: v1.talk,
                streaming_talk: None,
                singing_teacher: v1.singing_teacher,
                frame_decode: v1.frame_decode,
            },
        }
    }
}

pub(in super::super) type VoiceModelMetaV1 = Vec<CharacterMetaV1>;

#[derive(Deserialize)]
pub(in super::super) struct CharacterMetaV1 {
    name: String,
    styles: Vec<StyleMetaV1>,
    version: CharacterVersion,
    speaker_uuid: String,
    order: Option<u32>,
}

impl From<CharacterMetaV1> for CharacterMeta {
    fn from(v1: CharacterMetaV1) -> Self {
        Self {
            name: v1.name,
            styles: v1.styles.into_iter().map(Into::into).collect(),
            version: v1.version,
            speaker_uuid: v1.speaker_uuid,
            order: v1.order,
        }
    }
}

#[derive(Deserialize)]
struct StyleMetaV1 {
    id: StyleId,
    name: String,
    #[serde(default)]
    r#type: StyleTypeV1,
    order: Option<u32>,
}

impl From<StyleMetaV1> for StyleMeta {
    fn from(value: StyleMetaV1) -> Self {
        Self {
            id: value.id,
            name: value.name,
            r#type: value.r#type.into(),
            order: value.order,
        }
    }
}

#[derive(Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StyleTypeV1 {
    #[default]
    Talk,
    SingingTeacher,
    FrameDecode,
    Sing,
}

impl From<StyleTypeV1> for StyleType {
    fn from(value: StyleTypeV1) -> Self {
        match value {
            StyleTypeV1::Talk => Self::Talk,
            StyleTypeV1::SingingTeacher => Self::SingingTeacher,
            StyleTypeV1::FrameDecode => Self::FrameDecode,
            StyleTypeV1::Sing => Self::Sing,
        }
    }
}
