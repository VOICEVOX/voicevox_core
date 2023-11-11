use enum_map::Enum;
use macros::{InferenceGroup, InferenceInputSignature, TryFromVecOutputTensor};
use ndarray::{Array0, Array1, Array2};

use super::{InferenceSignature, OutputTensor};

#[derive(Clone, Copy, Enum, InferenceGroup)]
pub(crate) enum InferenceKind {
    PredictDuration,
    PredictIntonation,
    Decode,
}

pub(crate) enum PredictDuration {}

impl InferenceSignature for PredictDuration {
    type Group = InferenceKind;
    type Input = PredictDurationInput;
    type Output = PredictDurationOutput;
    const KIND: InferenceKind = InferenceKind::PredictDuration;
}

#[derive(InferenceInputSignature)]
#[input_signature(Signature = PredictDuration)]
pub(crate) struct PredictDurationInput {
    pub(crate) phoneme: Array1<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

#[derive(TryFromVecOutputTensor)]
pub(crate) struct PredictDurationOutput {
    pub(crate) phoneme_length: Array1<f32>,
}

pub(crate) enum PredictIntonation {}

impl InferenceSignature for PredictIntonation {
    type Group = InferenceKind;
    type Input = PredictIntonationInput;
    type Output = PredictIntonationOutput;
    const KIND: InferenceKind = InferenceKind::PredictIntonation;
}

#[derive(InferenceInputSignature)]
#[input_signature(Signature = PredictIntonation)]
pub(crate) struct PredictIntonationInput {
    pub(crate) length: Array0<i64>,
    pub(crate) vowel_phoneme: Array1<i64>,
    pub(crate) consonant_phoneme: Array1<i64>,
    pub(crate) start_accent: Array1<i64>,
    pub(crate) end_accent: Array1<i64>,
    pub(crate) start_accent_phrase: Array1<i64>,
    pub(crate) end_accent_phrase: Array1<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

#[derive(TryFromVecOutputTensor)]
pub(crate) struct PredictIntonationOutput {
    pub(crate) f0_list: Array1<f32>,
}

pub(crate) enum Decode {}

impl InferenceSignature for Decode {
    type Group = InferenceKind;
    type Input = DecodeInput;
    type Output = DecodeOutput;
    const KIND: InferenceKind = InferenceKind::Decode;
}

#[derive(InferenceInputSignature)]
#[input_signature(Signature = Decode)]
pub(crate) struct DecodeInput {
    pub(crate) f0: Array2<f32>,
    pub(crate) phoneme: Array2<f32>,
    pub(crate) speaker_id: Array1<i64>,
}

#[derive(TryFromVecOutputTensor)]
pub(crate) struct DecodeOutput {
    pub(crate) wave: Array1<f32>,
}
