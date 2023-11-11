use enum_map::Enum;
use macros::{InferenceInputSignature, TryFromVecOutputTensor};
use ndarray::{Array0, Array1, Array2};

use super::{InferenceGroup, InferenceSignature, OutputTensor};

pub(crate) enum InferenceGroupImpl {}

impl InferenceGroup for InferenceGroupImpl {
    type Kind = InferencelKindImpl;
}

#[derive(Clone, Copy, Enum)]
pub(crate) enum InferencelKindImpl {
    PredictDuration,
    PredictIntonation,
    Decode,
}

pub(crate) enum PredictDuration {}

impl InferenceSignature for PredictDuration {
    type Group = InferenceGroupImpl;
    type Input = PredictDurationInput;
    type Output = PredictDurationOutput;
    const INFERENCE: InferencelKindImpl = InferencelKindImpl::PredictDuration;
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
    type Group = InferenceGroupImpl;
    type Input = PredictIntonationInput;
    type Output = PredictIntonationOutput;
    const INFERENCE: InferencelKindImpl = InferencelKindImpl::PredictIntonation;
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
    type Group = InferenceGroupImpl;
    type Input = DecodeInput;
    type Output = DecodeOutput;
    const INFERENCE: InferencelKindImpl = InferencelKindImpl::Decode;
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
