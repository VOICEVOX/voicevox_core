//! `vvm_format_version=2`での`Manifest`と`CharacterMeta`。

use serde::Deserialize;

use crate::{
    CharacterMeta, VoiceModelId,
    core::{
        infer::domains::{
            FrameDecodeDomain, InferenceDomainMap, SingingTeacherV1Domain, StreamingTalkDomain,
            TalkDomain,
        },
        voice_model::manifest::{ManifestBody, ManifestDomain},
    },
};

#[derive(Deserialize)]
pub(in super::super) struct ManifestBodyV2 {
    id: VoiceModelId,
    metas_filename: String,
    talk: Option<ManifestDomain<TalkDomain>>,
    streaming_talk: Option<ManifestDomain<StreamingTalkDomain>>,
    singing_teacher: Option<ManifestDomain<SingingTeacherV1Domain>>,
    frame_decode: Option<ManifestDomain<FrameDecodeDomain>>,
}

impl From<ManifestBodyV2> for ManifestBody {
    fn from(v1: ManifestBodyV2) -> Self {
        Self {
            id: v1.id,
            metas_filename: v1.metas_filename,
            domains: InferenceDomainMap {
                talk: v1.talk,
                streaming_talk: v1.streaming_talk,
                singing_teacher_v1: v1.singing_teacher,
                singing_teacher_v2: None,
                frame_decode: v1.frame_decode,
            },
        }
    }
}

pub(in super::super) type VoiceModelMetaV2 = Vec<CharacterMeta>;
