use enum_map::Enum;
use macros::{InferenceDomain, InferenceInputSignature, InferenceOutputSignature};
use ndarray::{Array0, Array1, Array2};

use super::{InferenceInputSignature as _, InferenceOutputSignature as _, OutputTensor};

#[derive(Clone, Copy, Enum, InferenceDomain)]
pub(crate) enum InferenceKind {
    #[inference_domain(
        type Input = PredictDurationInput;
        type Output = PredictDurationOutput;
    )]
    PredictDuration,

    #[inference_domain(
        type Input = PredictIntonationInput;
        type Output = PredictIntonationOutput;
    )]
    PredictIntonation,

    #[inference_domain(
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
