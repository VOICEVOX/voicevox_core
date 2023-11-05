use std::sync::Arc;

use ndarray::{Array0, Array1, Array2};

use crate::infer::{InferenceRuntime, RunBuilder, Signature, TypedSession};

pub(crate) struct SessionSet<R: InferenceRuntime> {
    pub(crate) predict_duration: Arc<std::sync::Mutex<TypedSession<R, PredictDuration>>>,
    pub(crate) predict_intonation: Arc<std::sync::Mutex<TypedSession<R, PredictIntonation>>>,
    pub(crate) decode: Arc<std::sync::Mutex<TypedSession<R, Decode>>>,
}

pub(crate) struct PredictDuration {
    pub(crate) phoneme: Array1<i64>,
    pub(crate) speaker_id: Array1<i64>,
}

impl Signature for PredictDuration {
    type SessionSet<R: InferenceRuntime> = SessionSet<R>;
    type Output = (Vec<f32>,);

    fn get_session<R: InferenceRuntime>(
        session_set: &Self::SessionSet<R>,
    ) -> &Arc<std::sync::Mutex<TypedSession<R, Self>>> {
        &session_set.predict_duration
    }

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
    type SessionSet<R: InferenceRuntime> = SessionSet<R>;
    type Output = (Vec<f32>,);

    fn get_session<R: InferenceRuntime>(
        session_set: &Self::SessionSet<R>,
    ) -> &Arc<std::sync::Mutex<TypedSession<R, Self>>> {
        &session_set.predict_intonation
    }

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
    type SessionSet<R: InferenceRuntime> = SessionSet<R>;
    type Output = (Vec<f32>,);

    fn get_session<R: InferenceRuntime>(
        session_set: &Self::SessionSet<R>,
    ) -> &Arc<std::sync::Mutex<TypedSession<R, Self>>> {
        &session_set.decode
    }

    fn input<'a, 'b>(self, ctx: &'a mut impl RunBuilder<'b>) {
        ctx.input(self.f0)
            .input(self.phoneme)
            .input(self.speaker_id);
    }
}
