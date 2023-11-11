use anyhow::ensure;
use enum_map::Enum;
use ndarray::{Array0, Array1, Array2};

use super::{
    AnyTensor, InferenceGroup, InferenceInputSignature, InferenceRuntime, InferenceSignature,
    RunContextExt as _,
};

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

pub(crate) struct PredictDurationInput {
    pub(crate) phoneme: Array1<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

impl InferenceInputSignature for PredictDurationInput {
    type Signature = PredictDuration;

    fn make_run_context<R: InferenceRuntime>(self, sess: &mut R::Session) -> R::RunContext<'_> {
        R::RunContext::from(sess)
            .with_input(self.phoneme)
            .with_input(self.speaker_id)
    }
}

pub(crate) struct PredictDurationOutput {
    pub(crate) phoneme_length: Array1<f32>,
}

impl TryFrom<Vec<AnyTensor>> for PredictDurationOutput {
    type Error = anyhow::Error;

    fn try_from(tensors: Vec<AnyTensor>) -> Result<Self, Self::Error> {
        ensure!(
            tensors.len() == 1,
            "expected 1 tensor(s), got {}",
            tensors.len(),
        );

        let mut tensors = tensors.into_iter();
        let this = Self {
            phoneme_length: tensors
                .next()
                .expect("the length should have been checked")
                .try_into()?,
        };
        Ok(this)
    }
}

pub(crate) enum PredictIntonation {}

impl InferenceSignature for PredictIntonation {
    type Group = InferenceGroupImpl;
    type Input = PredictIntonationInput;
    type Output = PredictIntonationOutput;
    const INFERENCE: InferencelKindImpl = InferencelKindImpl::PredictIntonation;
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

    fn make_run_context<R: InferenceRuntime>(self, sess: &mut R::Session) -> R::RunContext<'_> {
        R::RunContext::from(sess)
            .with_input(self.length)
            .with_input(self.vowel_phoneme)
            .with_input(self.consonant_phoneme)
            .with_input(self.start_accent)
            .with_input(self.end_accent)
            .with_input(self.start_accent_phrase)
            .with_input(self.end_accent_phrase)
            .with_input(self.speaker_id)
    }
}

pub(crate) struct PredictIntonationOutput {
    pub(crate) f0_list: Array1<f32>,
}

impl TryFrom<Vec<AnyTensor>> for PredictIntonationOutput {
    type Error = anyhow::Error;

    fn try_from(tensors: Vec<AnyTensor>) -> Result<Self, Self::Error> {
        ensure!(
            tensors.len() == 1,
            "expected 1 tensor(s), got {}",
            tensors.len(),
        );

        let mut tensors = tensors.into_iter();
        let this = Self {
            f0_list: tensors
                .next()
                .expect("the length should have been checked")
                .try_into()?,
        };
        Ok(this)
    }
}

pub(crate) enum Decode {}

impl InferenceSignature for Decode {
    type Group = InferenceGroupImpl;
    type Input = DecodeInput;
    type Output = DecodeOutput;
    const INFERENCE: InferencelKindImpl = InferencelKindImpl::Decode;
}

pub(crate) struct DecodeInput {
    pub(crate) f0: Array2<f32>,
    pub(crate) phoneme: Array2<f32>,
    pub(crate) speaker_id: Array1<i64>,
}

impl InferenceInputSignature for DecodeInput {
    type Signature = Decode;

    fn make_run_context<R: InferenceRuntime>(self, sess: &mut R::Session) -> R::RunContext<'_> {
        R::RunContext::from(sess)
            .with_input(self.f0)
            .with_input(self.phoneme)
            .with_input(self.speaker_id)
    }
}

pub(crate) struct DecodeOutput {
    pub(crate) wave: Array1<f32>,
}

impl TryFrom<Vec<AnyTensor>> for DecodeOutput {
    type Error = anyhow::Error;

    fn try_from(tensors: Vec<AnyTensor>) -> Result<Self, Self::Error> {
        ensure!(
            tensors.len() == 1,
            "expected 1 tensor(s), got {}",
            tensors.len(),
        );

        let mut tensors = tensors.into_iter();
        let this = Self {
            wave: tensors
                .next()
                .expect("the length should have been checked")
                .try_into()?,
        };
        Ok(this)
    }
}
