//! 推論の出力の後処理。

use anyhow::anyhow;
use easy_ext::ext;
use itertools::{Itertools as _, chain};
use ndarray::{Array, Array1, Dim, Ix, RemoveAxis};
use typed_floats::{NonNaNFinite, PositiveFinite};

use crate::error::ErrorRepr;

// TODO: typed_floatsにissueかPRを出しに行き、スライス変換かbytemuck対応を入れてもらう
/// 推論結果の`[f32]`を`[NonNaNFinite<f32>]`として解釈する。
///
/// # Errors
///
/// NaNと±infに対して[ErrorRepr::RunModel]。
pub(crate) fn ensure_non_nan_finite(
    xs: &[f32],
    error: fn(&str) -> anyhow::Error,
) -> crate::Result<Vec<NonNaNFinite<f32>>> {
    xs.iter()
        .copied()
        .map(TryInto::try_into)
        .collect::<std::result::Result<_, _>>()
        .map_err(|_| {
            let invalid = &chain!(
                xs.iter().copied().any(f32::is_nan).then_some("NaN"),
                xs.contains(&f32::INFINITY).then_some("+inf"),
                xs.contains(&-f32::INFINITY).then_some("-inf"),
            )
            .join(", ");
            assert!(!invalid.is_empty());
            ErrorRepr::RunModel {
                note: None,
                source: error(invalid),
            }
            .into()
        })
}

// FIXME: 推論結果の`-0.0`は`+0.0`と読み替えてもよいのではないか？
// TODO: typed_floatsにissueかPRを出しに行き、スライス変換かbytemuck対応を入れてもらう
/// 推論結果の`[f32]`を`[PositiveFinite<f32>]`として解釈する。
///
/// # Errors
///
/// NaN、+inf、負値（≦ -0.0）に対して[ErrorRepr::RunModel]。
pub(crate) fn ensure_positive_finite(
    xs: &[f32],
    error: fn(&str) -> anyhow::Error,
) -> crate::Result<Vec<PositiveFinite<f32>>> {
    xs.iter()
        .copied()
        .map(TryInto::try_into)
        .collect::<std::result::Result<_, _>>()
        .map_err(|_| {
            let invalid = &chain!(
                xs.iter().copied().any(f32::is_nan).then_some("NaN"),
                xs.contains(&f32::INFINITY).then_some("+inf"),
                xs.iter()
                    .copied()
                    .any(f32::is_sign_negative)
                    .then_some("negative (≦ -0.0) values"),
            )
            .join(", ");
            assert!(!invalid.is_empty());
            ErrorRepr::RunModel {
                note: None,
                source: error(invalid),
            }
            .into()
        })
}

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
        if let Some(offset) = offset
            && offset != 0
        {
            unimplemented!("offset = {offset}");
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
                let source = anyhow!("could not squeeze a {orig_shape:?} array into a 1D one")
                    .context("unexpected output shape");
                ErrorRepr::RunModel { note: None, source }.into()
            })
    }
}
