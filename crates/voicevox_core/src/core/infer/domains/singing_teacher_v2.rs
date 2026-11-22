use std::{collections::BTreeSet, sync::LazyLock};

use enum_map::Enum;
use macros::{InferenceInputSignature, InferenceOperation, InferenceOutputSignature};
use ndarray::{Array1, Array2};
use serde::Deserialize;

use crate::StyleType;

use super::super::{
    InferenceDomain, InferenceInputSignature as _, InferenceOutputSignature as _, OutputTensor,
};

pub(crate) enum SingingTeacherV2Domain {}

impl InferenceDomain for SingingTeacherV2Domain {
    type Operation = SingingTeacherV2Operation;

    fn style_types() -> &'static BTreeSet<StyleType> {
        static STYLE_TYPES: LazyLock<BTreeSet<StyleType>> =
            LazyLock::new(|| [StyleType::SingingTeacher, StyleType::Sing].into());
        &STYLE_TYPES
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Enum, InferenceOperation)]
#[serde(rename_all = "snake_case")]
#[inference_operation(
    type Domain = SingingTeacherV2Domain;
)]
pub(crate) enum SingingTeacherV2Operation {
    #[inference_operation(
        type Input = PredictSingConsonantLengthInput;
        type Output = PredictSingConsonantLengthOutput;
    )]
    PredictSingConsonantLength,

    #[inference_operation(
        type Input = PredictSingF0V2Input;
        type Output = PredictSingF0V2Output;
    )]
    PredictSingF0V2,

    #[inference_operation(
        type Input = PredictSingVolumeV2Input;
        type Output = PredictSingVolumeV2Output;
    )]
    PredictSingVolumeV2,
}

#[derive(InferenceInputSignature)]
#[inference_input_signature(
    type Signature = PredictSingConsonantLength;
)]
pub(crate) struct PredictSingConsonantLengthInput {
    pub(crate) consonants: Array2<i64>,
    pub(crate) vowels: Array2<i64>,
    pub(crate) note_durations: Array2<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

#[derive(InferenceOutputSignature)]
pub(crate) struct PredictSingConsonantLengthOutput {
    pub(crate) consonant_lengths: Array2<i64>,
}

#[derive(InferenceInputSignature)]
#[inference_input_signature(
    type Signature = PredictSingF0V2;
)]
pub(crate) struct PredictSingF0V2Input {
    pub(crate) phonemes: Array2<i64>,
    pub(crate) notes: Array2<i64>,
    pub(crate) speaker_id: Array1<i64>,
    pub(crate) initial_noises: Array2<f32>,
}

#[derive(InferenceOutputSignature)]
pub(crate) struct PredictSingF0V2Output {
    pub(crate) f0s: Array2<f32>,
}

#[derive(InferenceInputSignature)]
#[inference_input_signature(
    type Signature = PredictSingVolumeV2;
)]
pub(crate) struct PredictSingVolumeV2Input {
    pub(crate) phonemes: Array2<i64>,
    pub(crate) notes: Array2<i64>,
    pub(crate) frame_f0s: Array2<f32>,
    pub(crate) speaker_id: Array1<i64>,
    pub(crate) initial_noises: Array2<f32>,
}

#[derive(InferenceOutputSignature)]
pub(crate) struct PredictSingVolumeV2Output {
    pub(crate) volumes: Array2<f32>,
}
