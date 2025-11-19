use std::{collections::BTreeSet, sync::LazyLock};

use enum_map::Enum;
use macros::{InferenceInputSignature, InferenceOperation, InferenceOutputSignature};
use ndarray::{Array1, Array2};
use serde::Deserialize;

use crate::StyleType;

use super::super::{
    super::manifest::SingingTeacherManifest, InferenceDomain, InferenceInputSignature as _,
    InferenceOutputSignature as _, OutputTensor,
};

pub(crate) enum SingingTeacherDomain {}

impl InferenceDomain for SingingTeacherDomain {
    type Operation = SingingTeacherOperation;
    type Manifest = SingingTeacherManifest;

    fn style_types() -> &'static BTreeSet<StyleType> {
        static STYLE_TYPES: LazyLock<BTreeSet<StyleType>> =
            LazyLock::new(|| [StyleType::SingingTeacher, StyleType::Sing].into());
        &STYLE_TYPES
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Enum, InferenceOperation)]
#[serde(rename_all = "snake_case")]
#[inference_operation(
    type Domain = SingingTeacherDomain;
)]
pub(crate) enum SingingTeacherOperation {
    #[inference_operation(
        type Input = PredictSingConsonantLengthInput;
        type Output = PredictSingConsonantLengthOutput;
    )]
    PredictSingConsonantLength,

    #[inference_operation(
        type Input = PredictSingF0Input;
        type Output = PredictSingF0Output;
    )]
    PredictSingF0,

    #[inference_operation(
        type Input = PredictSingVolumeInput;
        type Output = PredictSingVolumeOutput;
    )]
    PredictSingVolume,
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
    type Signature = PredictSingF0;
)]
pub(crate) struct PredictSingF0Input {
    pub(crate) phonemes: Array2<i64>,
    pub(crate) notes: Array2<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

#[derive(InferenceOutputSignature)]
pub(crate) struct PredictSingF0Output {
    pub(crate) f0s: Array2<f32>,
}

#[derive(InferenceInputSignature)]
#[inference_input_signature(
    type Signature = PredictSingVolume;
)]
pub(crate) struct PredictSingVolumeInput {
    pub(crate) phonemes: Array2<i64>,
    pub(crate) notes: Array2<i64>,
    pub(crate) frame_f0s: Array2<f32>,
    pub(crate) speaker_id: Array1<i64>,
}

#[derive(InferenceOutputSignature)]
pub(crate) struct PredictSingVolumeOutput {
    pub(crate) volumes: Array2<f32>,
}
