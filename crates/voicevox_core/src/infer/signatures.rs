use enum_map::Enum;
use ndarray::{Array0, Array1, Array2};

use crate::infer::{RunBuilder, Signature};

#[derive(Clone, Copy, Enum)]
pub(crate) enum SignatureKind {
    PredictDuration,
    PredictIntonation,
    Decode,
}

pub(crate) struct PredictDuration {
    pub(crate) phoneme: Array1<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

impl Signature for PredictDuration {
    type Kind = SignatureKind;
    type Output = (Vec<f32>,);

    const KIND: Self::Kind = SignatureKind::PredictDuration;

    fn input<'a, 'b>(self, ctx: &'a mut impl RunBuilder<'b>) {
        ctx.input(self.phoneme).input(self.speaker_id);
    }
}

pub(crate) struct PredictIntonation {
    pub(crate) length: Array0<i64>,
    pub(crate) vowel_phoneme: Array1<i64>,
    pub(crate) consonant_phoneme: Array1<i64>,
    pub(crate) start_accent: Array1<i64>,
    pub(crate) end_accent: Array1<i64>,
    pub(crate) start_accent_phrase: Array1<i64>,
    pub(crate) end_accent_phrase: Array1<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

impl Signature for PredictIntonation {
    type Kind = SignatureKind;
    type Output = (Vec<f32>,);

    const KIND: Self::Kind = SignatureKind::PredictIntonation;

    fn input<'a, 'b>(self, ctx: &'a mut impl RunBuilder<'b>) {
        ctx.input(self.length)
            .input(self.vowel_phoneme)
            .input(self.consonant_phoneme)
            .input(self.start_accent)
            .input(self.end_accent)
            .input(self.start_accent_phrase)
            .input(self.end_accent_phrase)
            .input(self.speaker_id);
    }
}

pub(crate) struct Decode {
    pub(crate) f0: Array2<f32>,
    pub(crate) phoneme: Array2<f32>,
    pub(crate) speaker_id: Array1<i64>,
}

impl Signature for Decode {
    type Kind = SignatureKind;
    type Output = (Vec<f32>,);

    const KIND: Self::Kind = SignatureKind::Decode;

    fn input<'a, 'b>(self, ctx: &'a mut impl RunBuilder<'b>) {
        ctx.input(self.f0)
            .input(self.phoneme)
            .input(self.speaker_id);
    }
}
