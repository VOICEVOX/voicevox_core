use std::num::NonZero;

use crate::FRAME_RATE;

#[derive(Clone, Copy, Debug)]
pub(crate) struct AudioSegmentation {
    pub(crate) start_offset_in_secs: f64,
    pub(crate) segment_length_in_secs: f64,
}

// FIXME: 受理する範囲についてテストを書く
impl AudioSegmentation {
    /// `start_offset`, `segment_length`が`frame_length`に対して受理してよければフレーム数に変換し、そうでなければパニックする。
    ///
    /// # Panics
    ///
    /// - [`Self::try_to_frames`]が失敗したらパニックする。
    #[track_caller]
    pub(crate) fn to_frames(self, frame_length: usize) -> (usize, NonZero<usize>) {
        match self.try_to_frames(frame_length) {
            Ok(frames) => frames,
            Err(err) => panic!("{err}"),
        }
    }

    /// `start_offset`, `segment_length`が`frame_length`に対して受理してよければフレーム数に変換し、そうでなければエラーを返す。
    ///
    /// # Errors
    ///
    /// - <code>[start_offset_in_secs] * [FRAME_RATE]</code>が`0.0`以上`frame_length`以下でなければ[`InvalidAudioSegmentationError::StartOffsetForFrameLength`]。
    /// - <code>[segment_length_in_secs] * [FRAME_RATE]</code>が`0.5`を超えていなければ[`InvalidAudioSegmentationError::SegmentLength`]。
    ///
    /// [start_offset_in_secs]: Self::start_offset_in_secs
    /// [segment_length_in_secs]: Self::segment_length_in_secs
    fn try_to_frames(
        self,
        frame_length: usize,
    ) -> Result<(usize, NonZero<usize>), InvalidAudioSegmentationError> {
        let start_offset_in_frames = to_frame_length(self.start_offset_in_secs);
        let start_offset_in_frames = if ((0.)..=frame_length as _).contains(&start_offset_in_frames)
        {
            start_offset_in_frames.round_ties_even() as _
        } else {
            return Err(InvalidAudioSegmentationError::StartOffsetOutOfBounds { frame_length });
        };

        #[expect(clippy::cast_nan_to_int)]
        const _: () = assert!(f64::NAN as usize == 0);
        const _: () = assert!(-1.0 as usize == 0);
        let Ok(segment_length_in_frame) =
            (to_frame_length(self.segment_length_in_secs).round_ties_even() as usize).try_into()
        else {
            return Err(InvalidAudioSegmentationError::SegmentLengthTooShort);
        };

        return Ok((start_offset_in_frames, segment_length_in_frame));

        const fn to_frame_length(secs: f64) -> f64 {
            secs * FRAME_RATE
        }
    }
}

impl Default for AudioSegmentation {
    fn default() -> Self {
        Self {
            start_offset_in_secs: 0.0,
            segment_length_in_secs: 0.3,
        }
    }
}

#[derive(derive_more::Display)]
enum InvalidAudioSegmentationError {
    #[display(
        "`start_offset` in frames must be positive and must not exceed the length of the audio \
         ({frame_length} frames; approx. {approx_secs} seconds)",
        approx_secs = (*frame_length as f64 / FRAME_RATE) as f32,
    )]
    StartOffsetOutOfBounds { frame_length: usize },

    #[display("`segment_length * {FRAME_RATE}` must be greater than `0.5`")]
    SegmentLengthTooShort,
}
