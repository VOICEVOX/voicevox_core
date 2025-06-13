use std::{collections::BTreeSet, sync::LazyLock};

use enum_map::Enum;
use macros::{InferenceInputSignature, InferenceOperation, InferenceOutputSignature};
use ndarray::{Array1, Array2};
use serde::Deserialize;

use crate::StyleType;

use super::super::{
    super::manifest::FrameDecodeManifest, InferenceDomain, InferenceInputSignature as _,
    InferenceOutputSignature as _, OutputTensor,
};

pub(crate) enum FrameDecodeDomain {}

impl InferenceDomain for FrameDecodeDomain {
    type Operation = FrameDecodeOperation;
    type Manifest = FrameDecodeManifest;

    fn style_types() -> &'static BTreeSet<StyleType> {
        static STYLE_TYPES: LazyLock<BTreeSet<StyleType>> =
            LazyLock::new(|| [StyleType::FrameDecode, StyleType::Sing].into());
        &STYLE_TYPES
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Enum, InferenceOperation)]
#[serde(rename_all = "snake_case")]
#[inference_operation(
    type Domain = FrameDecodeDomain;
)]
pub(crate) enum FrameDecodeOperation {
    #[inference_operation(
        type Input = SfDecodeInput;
        type Output = SfDecodeOutput;
    )]
    SfDecode,
}

#[derive(InferenceInputSignature)]
#[inference_input_signature(
    type Signature = SfDecode;
)]
pub(crate) struct SfDecodeInput {
    pub(crate) frame_phonemes: Array2<i64>,
    pub(crate) frame_f0s: Array2<f32>,
    pub(crate) frame_volumes: Array2<f32>,
    pub(crate) speaker_id: Array1<i64>,
}

#[derive(InferenceOutputSignature)]
pub(crate) struct SfDecodeOutput {
    pub(crate) wav: Array2<f32>,
}
