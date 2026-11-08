use std::{
    any::Any,
    fmt::Debug,
    marker::PhantomData,
    mem,
    ops::{Bound, RangeTo},
    slice::{self, SliceIndex},
};

use duplicate::duplicate_item;
use easy_ext::ext;
use ref_cast::{RefCastCustom, ref_cast_custom};

use crate::{
    StyleId, assert::assert_send_sync, core::adjust::MARGIN, dim::AUDIO_FEATURE_BINS,
    engine::PcmOptions,
};

/// 音声の中間表現。
#[cfg_attr(doc, doc(alias = "VoicevoxAudioFeature"))]
#[derive(Clone, PartialEq, derive_more::Debug)]
pub struct AudioFeature {
    /// (フレーム数, 特徴数)の形を持つ音声特徴量。
    #[debug("_")]
    internal_state: Box<[[f32; AUDIO_FEATURE_BINS]]>,
    /// 生成時に指定したスタイル番号。
    style_id: StyleId,
    /// `[f32]`からPCMを作るときのオプション。
    pcm_options: PcmOptions,
}

impl AudioFeature {
    /// # Panics
    ///
    /// `internal_state.len()`が`2 * MARGIN`より小さいならパニックする。
    pub(crate) fn new(
        internal_state: Box<[[f32; AUDIO_FEATURE_BINS]]>,
        style_id: StyleId,
        pcm_options: PcmOptions,
    ) -> Self {
        if internal_state.len() < 2 * MARGIN {
            panic!("too short: {}", internal_state.len());
        }
        Self {
            internal_state,
            style_id,
            pcm_options,
        }
    }

    /// 総フレーム数。
    ///
    /// この`AudioFeature`を生成するときに用いた`AudioQuery`の[`frame_length`]に等しい。
    ///
    /// [`frame_length`]: crate::AudioQuery::frame_length
    #[cfg_attr(doc, doc(alias = "voicevox_audio_feature_len"))]
    pub const fn len(&self) -> usize {
        self.internal_state.len() - 2 * MARGIN
    }

    /// [長さ]が`0`かどうか。
    ///
    /// [長さ]: Self::len
    #[cfg_attr(doc, doc(alias = "voicevox_audio_feature_is_empty"))]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `range`に対応した[`AudioFeatureSlice`]を返す。
    ///
    /// # Panics
    ///
    /// この`AudioFeature`の[長さ]に対して`range`が範囲外の場合パニックする。
    ///
    /// [長さ]: Self::len
    #[cfg_attr(doc, doc(alias = "voicevox_audio_feature_slice"))]
    #[track_caller]
    pub fn slice<'a>(
        &'a self,
        range: impl for<'b> SliceIndex<[_Frame<'b>], Output = [_Frame<'b>]> + Clone + Debug + 'static,
    ) -> AudioFeatureSlice<'a> {
        AudioFeatureSlice::new(self).slice(range)
    }
}

/// ある`AudioFeature`上の区間。
#[cfg_attr(doc, doc(alias = "VoicevoxAudioFeatureSlice"))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AudioFeatureSlice<'a> {
    audio_feature: &'a AudioFeature,

    // TODO: MSRVが1.96になったら`std::range::Range`にする。
    /// Invariant: must be valid for `audio_feature`.
    range: (usize, usize),
}

impl<'a> AudioFeatureSlice<'a> {
    /// 与えられた[`AudioFeature`]の全体を表す`AudioFeatureSlice`を返す。
    #[cfg_attr(doc, doc(alias = "voicevox_audio_feature_slice_new"))]
    pub const fn new(audio_feature: &'a AudioFeature) -> Self {
        Self {
            audio_feature,
            range: (0, audio_feature.len()),
        }
    }

    /// 元の[`AudioFeature`]。
    #[cfg_attr(doc, doc(alias = "VoicevoxAudioFeatureSlice::audio_feature"))]
    pub const fn audio_feature(self) -> &'a AudioFeature {
        self.audio_feature
    }

    /// この`AudioFeatureSlice`の始端が、元の[`AudioFeature`]のどの箇所にあたるか。
    #[cfg_attr(doc, doc(alias = "VoicevoxAudioFeatureSlice::start_offset"))]
    pub const fn start_offset(self) -> usize {
        let (start, _) = self.range;
        start
    }

    /// この`AudioFeatureSlice`が持つフレーム数。
    #[cfg_attr(doc, doc(alias = "voicevox_audio_feature_slice_len"))]
    pub const fn len(self) -> usize {
        let (start, end) = self.range;
        end - start
    }

    /// [長さ]が`0`かどうか。
    ///
    /// [長さ]: Self::len
    #[cfg_attr(doc, doc(alias = "voicevox_audio_feature_slice_is_empty"))]
    pub const fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// この`AudioFeatureSlice`のサブスライスを返す。
    ///
    /// # Panics
    ///
    /// この`AudioFeatureSlice`の[長さ]に対して`range`が範囲外の場合パニックする。
    ///
    /// [長さ]: Self::len
    #[cfg_attr(doc, doc(alias = "voicevox_audio_feature_slice_slice"))]
    #[track_caller]
    pub fn slice(
        self,
        range: impl for<'b> SliceIndex<[_Frame<'b>], Output = [_Frame<'b>]> + Clone + Debug + 'static,
    ) -> Self {
        match self.try_slice(range) {
            Ok(slice) => slice,
            Err(err) => panic!("{err}"),
        }
    }

    pub(crate) const fn style_id(self) -> StyleId {
        self.audio_feature.style_id
    }

    pub(crate) const fn pcm_options(self) -> PcmOptions {
        self.audio_feature.pcm_options
    }

    /// 指定した音声区間に対応する特徴量を両端にマージンを追加した上で切り出す
    #[track_caller]
    pub(crate) fn segment_with_margin(self) -> ndarray::ArrayView2<'a, f32> {
        let (start, end) = self.range;
        (&self.audio_feature.internal_state[start..end + 2 * MARGIN]).into()
    }
}

impl<'a> From<&'a AudioFeature> for AudioFeatureSlice<'a> {
    fn from(audio_feature: &'a AudioFeature) -> Self {
        Self::new(audio_feature)
    }
}

#[ext(AudioFeatureSliceExt)]
impl AudioFeatureSlice<'_> {
    pub fn try_slice(
        self,
        range: impl for<'a> SliceIndex<[_Frame<'a>], Output = [_Frame<'a>]> + Clone + Debug + 'static,
    ) -> Result<Self, AudioFeatureOutOfBoundsError>
    where
        Self: Sized,
    {
        let std::ops::Range { start, end } = self.validate_range(range)?;
        let (offset, _) = self.range;
        Ok(AudioFeatureSlice {
            audio_feature: self.audio_feature,
            range: (offset + start, offset + end),
        })
    }

    /// この`AudioFeature`に対して有効な範囲を与えられた場合、それを[`std::ops::Range<usize>`]に変換する。
    pub fn validate_range(
        &self,
        range: impl for<'a> SliceIndex<[_Frame<'a>], Output = [_Frame<'a>]> + Clone + Debug + 'static,
    ) -> Result<std::ops::Range<usize>, AudioFeatureOutOfBoundsError> {
        let len = self.len();

        // 長ささえ合っていればどう切り取ってもよい
        let internal_state = _Frame::new_slice(&self.audio_feature.internal_state[MARGIN..][..len]);

        return internal_state
            .get(range.clone())
            .map(|slice| {
                if slice.is_empty() {
                    let i =
                        partition_point(len, |i| internal_state[..i].get(range.clone()).is_none());
                    i..i
                } else {
                    const _: () = assert!(mem::size_of::<_Frame<'_>>() > 0);
                    // SAFETY: No matter how `range` implements `SliceIndex`, `<[_Frame<'a>]>::get`
                    // cannot return outside non-empty `[_Frame<'a>]` for all `'a`.
                    let start =
                        unsafe { slice.as_ptr().offset_from_unsigned(internal_state.as_ptr()) };
                    let end = start + slice.len();
                    start..end
                }
            })
            .ok_or_else(|| {
                fn to_range_to(range: &dyn Any) -> Option<RangeTo<usize>> {
                    range.downcast_ref().cloned()
                }

                #[rustversion::since(1.96)]
                #[duplicate_item(
                    f Type;
                    [ to_range ] [ Range ];
                    [ to_range_from ] [ RangeFrom ];
                    [ to_range_to_inclusive ] [ RangeToInclusive ];
                )]
                fn f(range: &dyn Any) -> Option<std::ops::Type<usize>> {
                    range
                        .downcast_ref::<std::range::Type<usize>>()
                        .copied()
                        .map(Into::into)
                        .or_else(|| range.downcast_ref().cloned())
                }

                #[rustversion::since(1.95)]
                fn to_range_inclusive(range: &dyn Any) -> Option<std::ops::RangeInclusive<usize>> {
                    range
                        .downcast_ref::<std::range::RangeInclusive<usize>>()
                        .copied()
                        .map(Into::into)
                        .or_else(|| range.downcast_ref().cloned())
                }

                #[rustversion::before(1.96)]
                #[duplicate_item(
                    f Type;
                    [ to_range ] [ Range ];
                    [ to_range_from ] [ RangeFrom ];
                    [ to_range_to_inclusive ] [ RangeToInclusive ];
                )]
                fn f(range: &dyn Any) -> Option<std::ops::Type<usize>> {
                    range.downcast_ref().cloned()
                }

                #[rustversion::before(1.95)]
                fn to_range_inclusive(range: &dyn Any) -> Option<std::ops::RangeInclusive<usize>> {
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
                    } else if start > end + 1 {
                        let end = end + 1;
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
                                Bound::Excluded(start) if start == end => {
                                    // Rust 1.99時点で何故か以下のようなメッセージになるのを模す
                                    //
                                    // ```rs
                                    //  const XS: [(); 10] = [(); 10];
                                    //  const WHATEVER: usize = 5;
                                    //  let _ = &XS[(
                                    //      Bound::Excluded(WHATEVER),
                                    //      Bound::Excluded(WHATEVER),
                                    //  )];
                                    // ```
                                    //
                                    // ```console
                                    // range end index 5 out of range for slice of length 10
                                    // ```
                                    AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len }
                                }
                                Bound::Excluded(start) if start > len => {
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
            });

        fn partition_point(n: usize, p: impl Fn(usize) -> bool) -> usize {
            if n == 0 {
                return 0;
            }
            let mut base = 0;
            let mut size = n;
            while size > 1 {
                let half = size / 2;
                if p(base + half) {
                    base += half;
                }
                size -= half;
            }
            if p(base) { base + 1 } else { 0 }
        }
    }
}

assert_send_sync!(AudioFeature);

#[derive(RefCastCustom)]
#[repr(transparent)]
pub struct _Frame<'a> {
    inner: [f32; AUDIO_FEATURE_BINS],
    _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl _Frame<'_> {
    #[ref_cast_custom]
    const fn _proof(inner: &[f32; AUDIO_FEATURE_BINS]) -> &Self;

    const fn new_slice(internal_state: &[[f32; AUDIO_FEATURE_BINS]]) -> &[Self] {
        unsafe {
            // SAFETY: `#[repr(transparent)]`.
            slice::from_raw_parts(internal_state.as_ptr().cast::<Self>(), internal_state.len())
        }
    }
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
    use pretty_assertions::assert_eq;

    use crate::{core::adjust::MARGIN, dim::AUDIO_FEATURE_BINS, engine::PcmOptions};

    use super::{AudioFeature, AudioFeatureOutOfBoundsError, AudioFeatureSliceExt as _};

    #[test]
    fn test_slicing_half_closed() {
        const LEN: usize = 10;

        let feat = AudioFeature::new(
            [[0.; // whatever
            AUDIO_FEATURE_BINS]; LEN + 2 * MARGIN]
                .into(),
            0.into(),          // whatever
            PcmOptions::DUMMY, // whatever
        );
        assert_eq!(LEN, feat.len());

        for (i, j, k, l) in iproduct!(0..=LEN, 0..=LEN, 0..=LEN, 0..=LEN) {
            let expected = [(); LEN]
                .get(i..j)
                .and_then(|slice| slice.get(k..l))
                .map(|slice| (i + k, slice.len()));
            let actual = feat
                .slice(..)
                .try_slice(i..j)
                .and_then(|slice| slice.try_slice(k..l))
                .map(|slice| (slice.start_offset(), slice.len()))
                .ok();
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_validate_range_for_nonempty() {
        const LEN: usize = 10;

        let feat = AudioFeature::new(
            [[0.; // whatever
            AUDIO_FEATURE_BINS]; LEN + 2 * MARGIN]
                .into(),
            0.into(),          // whatever
            PcmOptions::DUMMY, // whatever
        );
        let feat = feat.slice(..);
        assert_eq!(LEN, feat.len());

        // `Range`
        for (start, end) in iproduct!(0..=LEN, 0..=LEN) {
            assert_eq!(
                if start <= end {
                    Ok(start..end)
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
            assert_eq!(Ok(start..LEN), feat.validate_range(start..));
        }
        let start = LEN + 1;
        assert_eq!(
            Err(AudioFeatureOutOfBoundsError::StartOutOfBounds { start, len: LEN }),
            feat.validate_range(start..),
        );

        // `RangeTo`
        for end in 0..=LEN {
            assert_eq!(Ok(0..end), feat.validate_range(..end));
        }
        let end = LEN + 1;
        assert_eq!(
            Err(AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len: LEN }),
            feat.validate_range(..end),
        );

        // `RangeInclusive`
        for (start, end) in iproduct!(0..=LEN, 0..LEN) {
            assert_eq!(
                if start <= end + 1 {
                    Ok(start..end + 1)
                } else {
                    Err(AudioFeatureOutOfBoundsError::StartExceedsEnd {
                        start,
                        end: end + 1,
                    })
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
            assert_eq!(Ok(0..end + 1), feat.validate_range(..=end));
        }
        let end = LEN;
        assert_eq!(
            Err(AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len: LEN }),
            feat.validate_range(..=end),
        );

        // `RangeFull`
        assert_eq!(Ok(0..LEN), feat.validate_range(..));

        // `(Bound<usize>, Bound<usize>)`
        let bounds = chain!(
            (0..=LEN + 1).map(Bound::Included),
            (0..=LEN + 1).map(Bound::Excluded),
            [Bound::Unbounded],
        );
        for (start, end) in iproduct!(bounds.clone(), bounds) {
            let actual = feat.validate_range((start, end));
            assert!(!matches!(
                actual,
                Err(AudioFeatureOutOfBoundsError::Unknown { .. })
            ));
            // 面倒なのでこれ以上のエラーの種別までは確認しない。
            assert_eq!(
                [(); LEN].get((start, end)).map(|slice| slice.len()),
                actual.ok().map(|range| range.len()),
            );
        }
    }

    #[test]
    fn test_validate_range_for_empty() {
        let feat = AudioFeature::new(
            [[0.;  // whatever
            AUDIO_FEATURE_BINS]; 2 * MARGIN]
                .into(),
            0.into(),          // whatever
            PcmOptions::DUMMY, // whatever
        );
        let feat = feat.slice(..);
        assert_eq!(0, feat.len());

        assert_eq!(Ok(0..0), feat.validate_range(..));
        assert_eq!(Ok(0..0), feat.validate_range(0..0));

        let end = 0;
        assert_eq!(
            Err(AudioFeatureOutOfBoundsError::EndOutOfBounds { end, len: 0 }),
            feat.validate_range(..=0),
        );
    }
}
