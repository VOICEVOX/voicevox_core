use enum_map::Enum;
use ndarray::{Array0, Array1, Array2};

use crate::infer::{
    InferenceInput, InferenceSignature, SupportsInferenceInputTensor,
    SupportsInferenceInputTensors, SupportsInferenceSignature,
};

pub(crate) trait SupportsAllSignatures:
    SupportsInferenceSignature<PredictDuration>
    + SupportsInferenceSignature<PredictIntonation>
    + SupportsInferenceSignature<Decode>
{
}

impl<
        R: SupportsInferenceSignature<PredictDuration>
            + SupportsInferenceSignature<PredictIntonation>
            + SupportsInferenceSignature<Decode>,
    > SupportsAllSignatures for R
{
}

#[derive(Clone, Copy, Enum)]
pub(crate) enum SignatureKind {
    PredictDuration,
    PredictIntonation,
    Decode,
}

pub(crate) enum PredictDuration {}

impl InferenceSignature for PredictDuration {
    type Kind = SignatureKind;
    type Input = PredictDurationInput;
    type Output = (Vec<f32>,);
    const KIND: Self::Kind = SignatureKind::PredictDuration;
}

pub(crate) struct PredictDurationInput {
    pub(crate) phoneme: Array1<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

impl InferenceInput for PredictDurationInput {
    type Signature = PredictDuration;
}

impl<R: SupportsInferenceInputTensor<Array1<i64>>>
    SupportsInferenceInputTensors<PredictDurationInput> for R
{
    fn input(ctx: &mut R::RunContext<'_>, input: PredictDurationInput) {
        R::input(ctx, input.phoneme);
        R::input(ctx, input.speaker_id);
    }
}

pub(crate) enum PredictIntonation {}

impl InferenceSignature for PredictIntonation {
    type Kind = SignatureKind;
    type Input = PredictIntonationInput;
    type Output = (Vec<f32>,);
    const KIND: Self::Kind = SignatureKind::PredictIntonation;
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

impl InferenceInput for PredictIntonationInput {
    type Signature = PredictIntonation;
}

impl<R: SupportsInferenceInputTensor<Array0<i64>> + SupportsInferenceInputTensor<Array1<i64>>>
    SupportsInferenceInputTensors<PredictIntonationInput> for R
{
    fn input(ctx: &mut R::RunContext<'_>, input: PredictIntonationInput) {
        R::input(ctx, input.length);
        R::input(ctx, input.vowel_phoneme);
        R::input(ctx, input.consonant_phoneme);
        R::input(ctx, input.start_accent);
        R::input(ctx, input.end_accent);
        R::input(ctx, input.start_accent_phrase);
        R::input(ctx, input.end_accent_phrase);
        R::input(ctx, input.speaker_id);
    }
}

pub(crate) enum Decode {}

impl InferenceSignature for Decode {
    type Kind = SignatureKind;
    type Input = DecodeInput;
    type Output = (Vec<f32>,);
    const KIND: Self::Kind = SignatureKind::Decode;
}

pub(crate) struct DecodeInput {
    pub(crate) f0: Array2<f32>,
    pub(crate) phoneme: Array2<f32>,
    pub(crate) speaker_id: Array1<i64>,
}

impl InferenceInput for DecodeInput {
    type Signature = Decode;
}

impl<R: SupportsInferenceInputTensor<Array1<i64>> + SupportsInferenceInputTensor<Array2<f32>>>
    SupportsInferenceInputTensors<DecodeInput> for R
{
    fn input(ctx: &mut R::RunContext<'_>, input: DecodeInput) {
        R::input(ctx, input.f0);
        R::input(ctx, input.phoneme);
        R::input(ctx, input.speaker_id);
    }
}
