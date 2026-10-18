use std::{
    any::Any, fmt::Debug, marker::PhantomData, mem::MaybeUninit, ops::Bound, slice::SliceIndex,
};

use duplicate::duplicate_item;
use easy_ext::ext;

use crate::{StyleId, assert::assert_send_sync, core::adjust::MARGIN, engine::PcmOptions};

/// 音声の中間表現。
#[cfg_attr(doc, doc(alias = "VoicevoxAudioFeature"))]
#[derive(Clone, PartialEq, derive_more::Debug)]
pub struct AudioFeature {
    /// (フレーム数, 特徴数)の形を持つ音声特徴量。
    #[debug("_")]
    internal_state: ndarray::Array2<f32>,
    /// 生成時に指定したスタイル番号。
    style_id: StyleId,
    /// `[f32]`からPCMを作るときのオプション。
    pcm_options: PcmOptions,
}

impl AudioFeature {
    /// # Panics
    ///
    /// `internal_state.nrows()`が`2 * MARGIN`より小さいならパニックする。
    pub(crate) fn new(
        internal_state: ndarray::Array2<f32>,
        style_id: StyleId,
        pcm_options: PcmOptions,
    ) -> Self {
        if internal_state.nrows() < 2 * MARGIN {
            panic!("too short: {}", internal_state.nrows());
        }
        Self {
            internal_state,
            style_id,
            pcm_options,
        }
    }

    /// workaround paddingを除いた音声特徴量のフレーム数。
    pub fn frame_length(&self) -> usize {
        self.internal_state.nrows() - 2 * MARGIN
    }

    /// 指定した音声区間に対応する特徴量を両端にマージンを追加した上で切り出す
    #[track_caller]
    pub(crate) fn crop_with_margin(
        &self,
        range: impl for<'a> SliceIndex<[_Frame<'a>], Output = [_Frame<'a>]> + Clone + Debug + 'static,
    ) -> ndarray::ArrayView2<'_, f32> {
        let std::ops::Range { start, end } = match self.validate_range(range) {
            Ok(MaybeEmptyRange::Empty) => 0..0,
            Ok(MaybeEmptyRange::NonEmpty(range)) => range,
            Err(err) => panic!("{err}"),
        };
        let range = start..end + 2 * MARGIN;
        self.internal_state.slice(ndarray::s![range, ..])
    }

    pub(crate) fn style_id(&self) -> StyleId {
        self.style_id
    }

    pub(crate) fn pcm_options(&self) -> PcmOptions {
        self.pcm_options
    }
}

// TODO: #1454をやるときに`__internal::interop`から出して使う

#[ext(AudioFeatureExt)]
impl AudioFeature {
    /// この`AudioFeature`に対して有効な範囲を与えられた場合、それを[`std::ops::Range<usize>`]に変換する。
    pub fn validate_range(
        &self,
        range: impl for<'a> SliceIndex<[_Frame<'a>], Output = [_Frame<'a>]> + Clone + Debug + 'static,
    ) -> Result<MaybeEmptyRange, AudioFeatureOutOfBoundsError> {
        let len = self.frame_length();
        let dummy_audio_feature = (0..len).map(|_| _Frame::DUMMY).collect::<Vec<_>>();

        dummy_audio_feature
            .get(range.clone())
            .map(|slice| {
                if slice.is_empty() {
                    MaybeEmptyRange::Empty
                } else {
                    // SAFETY: No matter how `range` implements `SliceIndex`, `<[_Frame<'a>]>::get`
                    // cannot return outside non-empty `[_Frame<'a>]` for all `'a`.
                    let start = unsafe { slice.as_ptr().offset_from(dummy_audio_feature.as_ptr()) }
                        as usize;
                    let end = start + slice.len();
                    MaybeEmptyRange::NonEmpty(start..end)
                }
            })
            .ok_or_else(|| {
                #[rustversion::since(1.96)]
                #[duplicate_item(
                    f Type;
                    [ to_range ] [ Range ];
                    [ to_range_from ] [ RangeFrom ];
                    [ to_range_inclusive ] [ RangeInclusive ];
                    [ to_range_to_inclusive ] [ RangeToInclusive ];
                )]
                fn f(range: &dyn Any) -> Option<std::ops::Type<usize>> {
                    range
                        .downcast_ref::<std::range::Type<usize>>()
                        .copied()
                        .map(Into::into)
                        .or_else(|| range.downcast_ref().cloned())
                }

                #[rustversion::since(1.98)]
                #[expect(clippy::useless_conversion, reason = "Clippy's bug??")]
                fn to_range_to(range: &dyn Any) -> Option<std::ops::RangeTo<usize>> {
                    range
                        .downcast_ref::<std::range::RangeTo<usize>>()
                        .copied()
                        .map(Into::into)
                        .or_else(|| range.downcast_ref().cloned())
                }

                #[rustversion::not(since(1.96))]
                #[duplicate_item(
                    f Type;
                    [ to_range ] [ Range ];
                    [ to_range_from ] [ RangeFrom ];
                    [ to_range_inclusive ] [ RangeInclusive ];
                    [ to_range_to_inclusive ] [ RangeToInclusive ];
                )]
                fn f(range: &dyn Any) -> Option<std::ops::Type<usize>> {
                    range.downcast_ref().cloned()
                }

                #[rustversion::before(1.98)]
                fn to_range_to(range: &dyn Any) -> Option<std::ops::RangeTo<usize>> {
                    range.downcast_ref().cloned()
                }

                let could_not_categorize = || unreachable!("could not categorize `{range:?}`");

                let error_for_range = |std::ops::Range { start, end }: std::ops::Range<usize>| {
                    if start > len {
                        AudioFeatureOutOfBoundsError::StartOutOfBounds { start, len }
                    } else if end > len {
                        AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len }
                    } else if start > end {
                        AudioFeatureOutOfBoundsError::StartExceedsEnd { start, end }
                    } else {
                        could_not_categorize()
                    }
                };

                let error_for_range_inclusive = |range: std::ops::RangeInclusive<usize>| {
                    let start = *range.start();
                    let end = *range.end();
                    if start > len {
                        AudioFeatureOutOfBoundsError::StartOutOfBounds { start, len }
                    } else if end >= len {
                        AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len }
                    } else if start > end {
                        AudioFeatureOutOfBoundsError::StartExceedsEnd { start, end }
                    } else {
                        could_not_categorize()
                    }
                };

                if let Some(range) = to_range(&range) {
                    error_for_range(range)
                } else if let Some(range) = to_range_from(&range) {
                    error_for_range(range.start..len)
                } else if let Some(range) = to_range_to(&range) {
                    error_for_range(0..range.end)
                } else if let Some(range) = to_range_inclusive(&range) {
                    error_for_range_inclusive(range)
                } else if let Some(range) = to_range_to_inclusive(&range) {
                    error_for_range_inclusive(0..=range.end)
                } else if let Some(&(start, end)) =
                    <dyn Any>::downcast_ref::<(Bound<usize>, Bound<usize>)>(&range)
                {
                    match end {
                        Bound::Included(end) if end >= len => {
                            AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len }
                        }
                        Bound::Excluded(end) if end > len => {
                            AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len }
                        }
                        _ => {
                            let end = match end {
                                Bound::Included(end) => end + 1,
                                Bound::Excluded(end) => end,
                                Bound::Unbounded => len,
                            };
                            match start {
                                Bound::Included(start) if start > len => {
                                    AudioFeatureOutOfBoundsError::StartOutOfBounds { start, len }
                                }
                                Bound::Included(start) if start > end => {
                                    AudioFeatureOutOfBoundsError::StartExceedsEnd { start, end }
                                }
                                Bound::Excluded(start) if start >= len => {
                                    AudioFeatureOutOfBoundsError::StartOutOfBounds { start, len }
                                }
                                Bound::Excluded(start) if start >= end => {
                                    AudioFeatureOutOfBoundsError::StartExceedsEnd { start, end }
                                }
                                _ => could_not_categorize(),
                            }
                        }
                    }
                } else {
                    AudioFeatureOutOfBoundsError::Unknown {
                        range: format!("{range:?}"),
                        len,
                    }
                }
            })
    }
}

assert_send_sync!(AudioFeature);

#[repr(transparent)]
pub struct _Frame<'a> {
    inner: MaybeUninit<[u8; 80]>,
    _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl _Frame<'_> {
    const DUMMY: Self = Self {
        inner: MaybeUninit::uninit(),
        _marker: PhantomData,
    };
}

#[derive(PartialEq, Debug)]
pub enum MaybeEmptyRange {
    Empty,
    NonEmpty(std::ops::Range<usize>),
}

#[derive(PartialEq, derive_more::Display, Debug)]
pub enum AudioFeatureOutOfBoundsError {
    #[display("range start index {start} out of range for audio feature of length {len}")]
    StartOutOfBounds { start: usize, len: usize },

    #[display("range end index {end} out of range for audio feature of length {len}")]
    EndOutOfBounds { end: usize, len: usize },

    #[display("range starts at {start} but ends at {end}")]
    StartExceedsEnd { start: usize, end: usize },

    #[display("range `{range}` out of bounds for audio feature of length {len}")]
    Unknown { range: String, len: usize },
}

#[cfg(test)]
mod tests {
    use std::ops::Bound;

    use itertools::{chain, iproduct};
    use ndarray::Array;
    use pretty_assertions::assert_eq;

    use crate::{core::adjust::MARGIN, engine::PcmOptions};

    use super::{
        AudioFeature, AudioFeatureExt as _, AudioFeatureOutOfBoundsError, MaybeEmptyRange,
    };

    #[test]
    fn test_validate_range_for_nonempty() {
        const LEN: usize = 10;

        let feat = AudioFeature::new(
            Array::from_shape_fn((LEN + 2 * MARGIN, 80), |_| 0.),
            0.into(),          // whatever
            PcmOptions::DUMMY, // whatever
        );

        // `Range`
        for (start, end) in iproduct!(0..=LEN, 0..=LEN) {
            assert_eq!(
                if start == end {
                    Ok(MaybeEmptyRange::Empty)
                } else if start < end {
                    Ok(MaybeEmptyRange::NonEmpty(start..end))
                } else {
                    Err(AudioFeatureOutOfBoundsError::StartExceedsEnd { start, end })
                },
                feat.validate_range(start..end),
            );
        }
        let end = LEN + 1;
        assert_eq!(
            Err(AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len: LEN }),
            feat.validate_range(0..end),
        );
        let start = LEN + 1;
        assert_eq!(
            Err(AudioFeatureOutOfBoundsError::StartOutOfBounds { start, len: LEN }),
            feat.validate_range(start..end),
        );

        // `RangeFrom`
        for start in 0..=LEN {
            assert_eq!(
                Ok(match start {
                    LEN => MaybeEmptyRange::Empty,
                    start => MaybeEmptyRange::NonEmpty(start..LEN),
                }),
                feat.validate_range(start..),
            );
        }
        let start = LEN + 1;
        assert_eq!(
            Err(AudioFeatureOutOfBoundsError::StartOutOfBounds { start, len: LEN }),
            feat.validate_range(start..),
        );

        // `RangeTo`
        for end in 0..=LEN {
            assert_eq!(
                Ok(match end {
                    0 => MaybeEmptyRange::Empty,
                    end => MaybeEmptyRange::NonEmpty(0..end),
                }),
                feat.validate_range(..end),
            );
        }
        let end = LEN + 1;
        assert_eq!(
            Err(AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len: LEN }),
            feat.validate_range(..end),
        );

        // `RangeInclusive`
        for (start, end) in iproduct!(0..=LEN, 0..LEN) {
            assert_eq!(
                if start == end + 1 {
                    Ok(MaybeEmptyRange::Empty)
                } else if start <= end {
                    Ok(MaybeEmptyRange::NonEmpty(start..end + 1))
                } else {
                    Err(AudioFeatureOutOfBoundsError::StartExceedsEnd { start, end })
                },
                feat.validate_range(start..=end),
            );
        }
        let end = LEN;
        assert_eq!(
            Err(AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len: LEN }),
            feat.validate_range(0..=end),
        );
        let start = LEN + 1;
        assert_eq!(
            Err(AudioFeatureOutOfBoundsError::StartOutOfBounds { start, len: LEN }),
            feat.validate_range(start..=end),
        );

        // `RangeToInclusive`
        for end in 0..LEN {
            assert_eq!(
                Ok(MaybeEmptyRange::NonEmpty(0..end + 1)),
                feat.validate_range(..=end),
            );
        }
        let end = LEN;
        assert_eq!(
            Err(AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len: LEN }),
            feat.validate_range(..=end),
        );

        // `RangeFull`
        assert_eq!(
            Ok(MaybeEmptyRange::NonEmpty(0..LEN)),
            feat.validate_range(..),
        );

        // `(Bound<usize>, Bound<usize>)`
        let bounds = chain!(
            (0..=LEN + 1).map(Bound::Included),
            (0..=LEN + 1).map(Bound::Excluded),
            [Bound::Unbounded],
        );
        for (start, end) in iproduct!(bounds.clone(), bounds) {
            // 面倒なのでエラーの種別までは確認しない。
            assert_eq!(
                [(); LEN].get((start, end)).map(|slice| slice.len()),
                feat.validate_range((start, end))
                    .ok()
                    .map(|range| match range {
                        MaybeEmptyRange::Empty => 0,
                        MaybeEmptyRange::NonEmpty(range) => range.len(),
                    }),
            );
        }
    }

    #[test]
    fn test_validate_range_for_empty() {
        let feat = AudioFeature::new(
            Array::from_shape_fn((2 * MARGIN, 80), |_| 0.),
            0.into(),          // whatever
            PcmOptions::DUMMY, // whatever
        );

        assert_eq!(Ok(MaybeEmptyRange::Empty), feat.validate_range(..));
        assert_eq!(Ok(MaybeEmptyRange::Empty), feat.validate_range(0..0));

        let end = 0;
        assert_eq!(
            Err(AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len: 0 }),
            feat.validate_range(..=0),
        );
    }
}
