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
    fn input(input: PredictDurationInput, ctx: &mut R::RunContext<'_>) {
        ctx.input(input.phoneme).input(input.speaker_id);
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
    fn input(input: PredictIntonationInput, ctx: &mut R::RunContext<'_>) {
        ctx.input(input.length)
            .input(input.vowel_phoneme)
            .input(input.consonant_phoneme)
            .input(input.start_accent)
            .input(input.end_accent)
            .input(input.start_accent_phrase)
            .input(input.end_accent_phrase)
            .input(input.speaker_id);
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
    fn input(input: DecodeInput, ctx: &mut R::RunContext<'_>) {
        ctx.input(input.f0)
            .input(input.phoneme)
            .input(input.speaker_id);
    }
}
