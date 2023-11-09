use enum_map::Enum;
use ndarray::{Array0, Array1, Array2};

use crate::infer::{
    InferenceInputSignature, InferenceSignature, RunContextExt as _,
    SupportsInferenceInputSignature, SupportsInferenceInputTensor,
};

#[derive(Clone, Copy, Enum)]
pub(crate) enum InferenceSignatureKind {
    PredictDuration,
    PredictIntonation,
    Decode,
}

pub(crate) enum PredictDuration {}

impl InferenceSignature for PredictDuration {
    type Kind = InferenceSignatureKind;
    type Input = PredictDurationInput;
    type Output = (Vec<f32>,);
    const KIND: Self::Kind = InferenceSignatureKind::PredictDuration;
}

pub(crate) struct PredictDurationInput {
    pub(crate) phoneme: Array1<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

impl InferenceInputSignature for PredictDurationInput {
    type Signature = PredictDuration;
}

impl<R: SupportsInferenceInputTensor<Array1<i64>>>
    SupportsInferenceInputSignature<PredictDurationInput> for R
{
    fn make_run_context(
        sess: &mut Self::Session,
        input: PredictDurationInput,
    ) -> Self::RunContext<'_> {
        Self::RunContext::from(sess)
            .with_input(input.phoneme)
            .with_input(input.speaker_id)
    }
}

pub(crate) enum PredictIntonation {}

impl InferenceSignature for PredictIntonation {
    type Kind = InferenceSignatureKind;
    type Input = PredictIntonationInput;
    type Output = (Vec<f32>,);
    const KIND: Self::Kind = InferenceSignatureKind::PredictIntonation;
}

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

impl InferenceInputSignature for PredictIntonationInput {
    type Signature = PredictIntonation;
}

impl<R: SupportsInferenceInputTensor<Array0<i64>> + SupportsInferenceInputTensor<Array1<i64>>>
    SupportsInferenceInputSignature<PredictIntonationInput> for R
{
    fn make_run_context(
        sess: &mut Self::Session,
        input: PredictIntonationInput,
    ) -> Self::RunContext<'_> {
        Self::RunContext::from(sess)
            .with_input(input.length)
            .with_input(input.vowel_phoneme)
            .with_input(input.consonant_phoneme)
            .with_input(input.start_accent)
            .with_input(input.end_accent)
            .with_input(input.start_accent_phrase)
            .with_input(input.end_accent_phrase)
            .with_input(input.speaker_id)
    }
}

pub(crate) enum Decode {}

impl InferenceSignature for Decode {
    type Kind = InferenceSignatureKind;
    type Input = DecodeInput;
    type Output = (Vec<f32>,);
    const KIND: Self::Kind = InferenceSignatureKind::Decode;
}

pub(crate) struct DecodeInput {
    pub(crate) f0: Array2<f32>,
    pub(crate) phoneme: Array2<f32>,
    pub(crate) speaker_id: Array1<i64>,
}

impl InferenceInputSignature for DecodeInput {
    type Signature = Decode;
}

impl<R: SupportsInferenceInputTensor<Array1<i64>> + SupportsInferenceInputTensor<Array2<f32>>>
    SupportsInferenceInputSignature<DecodeInput> for R
{
    fn make_run_context(sess: &mut Self::Session, input: DecodeInput) -> Self::RunContext<'_> {
        Self::RunContext::from(sess)
            .with_input(input.f0)
            .with_input(input.phoneme)
            .with_input(input.speaker_id)
    }
}
