use std::{collections::BTreeSet, sync::LazyLock};

use enum_map::Enum;
use macros::{InferenceInputSignature, InferenceOperation, InferenceOutputSignature};
use ndarray::{Array1, Array2};
use serde::Deserialize;

use crate::StyleType;

use super::super::{
    InferenceDomain, InferenceInputSignature as _, InferenceOutputSignature as _, OutputTensor,
};

pub(crate) enum SingingTeacherV1Domain {}

impl InferenceDomain for SingingTeacherV1Domain {
    type Operation = SingingTeacherV1Operation;

    fn style_types() -> &'static BTreeSet<StyleType> {
        static STYLE_TYPES: LazyLock<BTreeSet<StyleType>> =
            LazyLock::new(|| [StyleType::SingingTeacher, StyleType::Sing].into());
        &STYLE_TYPES
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Enum, InferenceOperation)]
#[serde(rename_all = "snake_case")]
#[inference_operation(
    type Domain = SingingTeacherV1Domain;
)]
pub(crate) enum SingingTeacherV1Operation {
    #[inference_operation(
        type Input = PredictSingConsonantLengthInput;
        type Output = PredictSingConsonantLengthOutput;
    )]
    PredictSingConsonantLength,

    #[serde(rename = "predict_sing_f0")]
    #[inference_operation(
        type Input = PredictSingF0V1Input;
        type Output = PredictSingF0V1Output;
    )]
    PredictSingF0V1,

    #[serde(rename = "predict_sing_volume")]
    #[inference_operation(
        type Input = PredictSingVolumeV1Input;
        type Output = PredictSingVolumeV1Output;
    )]
    PredictSingVolumeV1,
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
    type Signature = PredictSingF0V1;
)]
pub(crate) struct PredictSingF0V1Input {
    pub(crate) phonemes: Array2<i64>,
    pub(crate) notes: Array2<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

#[derive(InferenceOutputSignature)]
pub(crate) struct PredictSingF0V1Output {
    pub(crate) f0s: Array2<f32>,
}

#[derive(InferenceInputSignature)]
#[inference_input_signature(
    type Signature = PredictSingVolumeV1;
)]
pub(crate) struct PredictSingVolumeV1Input {
    pub(crate) phonemes: Array2<i64>,
    pub(crate) notes: Array2<i64>,
    pub(crate) frame_f0s: Array2<f32>,
    pub(crate) speaker_id: Array1<i64>,
}

#[derive(InferenceOutputSignature)]
pub(crate) struct PredictSingVolumeV1Output {
    pub(crate) volumes: Array2<f32>,
}
