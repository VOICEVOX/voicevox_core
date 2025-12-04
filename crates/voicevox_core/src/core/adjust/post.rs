//! 推論の出力の後処理。

use anyhow::anyhow;
use easy_ext::ext;
use ndarray::{Array, Array1, Dim, Ix, RemoveAxis};

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

#[ext(Array1ExtForPostProcess)]
impl<T> Array1<T> {
    pub(crate) fn into_vec(self) -> Vec<T> {
        let (vec, offset) = self.into_raw_vec_and_offset();
        // TODO: Rust 2024にしたらlet chainにする
        if let Some(offset) = offset {
            if offset != 0 {
                unimplemented!("offset = {offset}");
            }
        }
        vec
    }
}

#[ext(ArrayExt)]
impl<T, const N: usize> Array<T, Dim<[Ix; N]>>
where
    Dim<[Ix; N]>: RemoveAxis,
{
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
