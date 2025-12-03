//! 推論の出力の後処理。

use anyhow::anyhow;
use easy_ext::ext;
use ndarray::{Array, Array1, Dimension};

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

#[ext(ArrayExt)]
impl<T, D: Dimension> Array<T, D> {
    pub(crate) fn squeeze_into_1d(self) -> crate::Result<Array1<T>> {
        let orig_shape = self.dim();
        self.into_dyn()
            .squeeze()
            .into_dimensionality()
            .map_err(|_| {
                let sources = anyhow!("could not squeeze a {orig_shape:?} array into a 1D one")
                    .context("unexpected output shape");
                ErrorRepr::RunModel(sources).into()
            })
    }
}
