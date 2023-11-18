use easy_ext::ext;
use ndarray::{Array1, ArrayView1, ArrayView2};

use crate::{StyleId, Synthesizer};

#[ext(PerformInference)]
impl Synthesizer<()> {
    pub fn predict_duration(
        &self,
        phoneme_list: Array1<i64>,
        style_id: StyleId,
    ) -> crate::Result<Vec<f32>> {
        self.predict_duration(phoneme_list, style_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn predict_intonation(
        &self,
        vowel_phoneme_list: Array1<i64>,
        consonant_phoneme_list: Array1<i64>,
        start_accent_list: Array1<i64>,
        end_accent_list: Array1<i64>,
        start_accent_phrase_list: Array1<i64>,
        end_accent_phrase_list: Array1<i64>,
        style_id: StyleId,
    ) -> crate::Result<Vec<f32>> {
        self.predict_intonation(
            vowel_phoneme_list,
            consonant_phoneme_list,
            start_accent_list,
            end_accent_list,
            start_accent_phrase_list,
            end_accent_phrase_list,
            style_id,
        )
    }

    pub fn decode(
        &self,
        f0: ArrayView1<'_, f32>,
        phoneme: ArrayView2<'_, f32>,
        style_id: StyleId,
    ) -> crate::Result<Vec<f32>> {
        self.decode(f0, phoneme, style_id)
    }
}
