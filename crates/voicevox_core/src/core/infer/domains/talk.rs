use std::{collections::BTreeSet, sync::LazyLock};

use enum_map::Enum;
use macros::{InferenceInputSignature, InferenceOperation, InferenceOutputSignature};
use ndarray::{Array0, Array1, Array2};
use serde::Deserialize;

use crate::StyleType;

use super::super::{
    super::manifest::TalkManifest, InferenceDomain, InferenceInputSignature as _,
    InferenceOutputSignature as _, OutputTensor,
};

pub(crate) enum TalkDomain {}

impl InferenceDomain for TalkDomain {
    type Operation = TalkOperation;
    type Manifest = TalkManifest;

    fn style_types() -> &'static BTreeSet<StyleType> {
        static STYLE_TYPES: LazyLock<BTreeSet<StyleType>> =
            LazyLock::new(|| [StyleType::Talk].into());
        &STYLE_TYPES
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Enum, InferenceOperation)]
#[serde(rename_all = "snake_case")]
#[inference_operation(
    type Domain = TalkDomain;
)]
pub(crate) enum TalkOperation {
    #[inference_operation(
        type Input = PredictDurationInput;
        type Output = PredictDurationOutput;
    )]
    PredictDuration,

    #[inference_operation(
        type Input = PredictIntonationInput;
        type Output = PredictIntonationOutput;
    )]
    PredictIntonation,

    #[inference_operation(
        type Input = DecodeInput;
        type Output = DecodeOutput;
    )]
    Decode,
}

#[derive(InferenceInputSignature)]
#[inference_input_signature(
    type Signature = PredictDuration;
)]
pub(crate) struct PredictDurationInput {
    pub(crate) phoneme_list: Array1<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

#[derive(InferenceOutputSignature)]
pub(crate) struct PredictDurationOutput {
    pub(crate) phoneme_length: Array1<f32>,
}

#[derive(InferenceInputSignature)]
#[inference_input_signature(
    type Signature = PredictIntonation;
)]
pub(crate) struct PredictIntonationInput {
    pub(crate) length: Array0<i64>,
    pub(crate) vowel_phoneme_list: Array1<i64>,
    pub(crate) consonant_phoneme_list: Array1<i64>,
    pub(crate) start_accent_list: Array1<i64>,
    pub(crate) end_accent_list: Array1<i64>,
    pub(crate) start_accent_phrase_list: Array1<i64>,
    pub(crate) end_accent_phrase_list: Array1<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

#[derive(InferenceOutputSignature)]
pub(crate) struct PredictIntonationOutput {
    pub(crate) f0_list: Array1<f32>,
}

#[derive(InferenceInputSignature)]
#[inference_input_signature(
    type Signature = Decode;
)]
pub(crate) struct DecodeInput {
    pub(crate) f0: Array2<f32>,
    pub(crate) phoneme: Array2<f32>,
    pub(crate) speaker_id: Array1<i64>,
}

#[derive(InferenceOutputSignature)]
pub(crate) struct DecodeOutput {
    pub(crate) wave: Array1<f32>,
}
