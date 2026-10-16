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
    pub(crate) fn crop_with_margin(
        &self,
        range: std::ops::Range<usize>,
    ) -> ndarray::ArrayView2<'_, f32> {
        let frame_length = self.frame_length();
        let std::ops::Range { start, end } = range;
        if start > frame_length {
            panic!(
                "range start index {start} out of range for audio feature of length {frame_length}",
            );
        }
        if end > frame_length {
            panic!("range end index {end} out of range for audio feature of length {frame_length}");
        }
        if start > end {
            panic!("range starts at {start} but ends at {end}");
        }
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

assert_send_sync!(AudioFeature);
