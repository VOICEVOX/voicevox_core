//! 推論の出力の後処理。

use easy_ext::ext;
use ndarray::{Array1, Array2};

use crate::error::ErrorRepr;

pub(crate) fn ensure_minimum_phoneme_length(mut output: Vec<f32>) -> Vec<f32> {
    const PHONEME_LENGTH_MINIMAL: f32 = 0.01;

    for output_item in output.iter_mut() {
        if *output_item < PHONEME_LENGTH_MINIMAL {
            *output_item = PHONEME_LENGTH_MINIMAL;
        }
    }
    output
}

#[ext(Array2Ext)]
impl<T> Array2<T> {
    pub(crate) fn squeeze_into_1d(self) -> crate::Result<Array1<T>> {
        self.into_dyn()
            .squeeze()
            .into_dimensionality()
            .map_err(|err| {
                let err = anyhow::Error::from(err).context("unexpected output shape");
                ErrorRepr::RunModel(err).into()
            })
    }
}
