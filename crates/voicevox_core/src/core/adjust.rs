pub(crate) fn ensure_minimum_phoneme_length(mut output: Vec<f32>) -> Vec<f32> {
    const PHONEME_LENGTH_MINIMAL: f32 = 0.01;

    for output_item in output.iter_mut() {
        if *output_item < PHONEME_LENGTH_MINIMAL {
            *output_item = PHONEME_LENGTH_MINIMAL;
        }
    }
    output
}

/// 音が途切れてしまうのを避けるworkaround処理
// TODO: 改善したらここのpadding処理を取り除く
pub(crate) fn pad_decoder_feature(
    f0: ndarray::Array1<f32>,
    phoneme: ndarray::Array2<f32>,
) -> (usize, ndarray::Array1<f32>, ndarray::Array2<f32>) {
    /// 音が途切れてしまうのを避けるworkaround処理のためのパディング幅（フレーム数）
    const PADDING_FRAME_LENGTH: usize = 38; // (0.4秒 * 24000Hz / 256.0).round()

    let start_and_end_padding_size = 2 * PADDING_FRAME_LENGTH;
    let length_with_padding = f0.len() + start_and_end_padding_size;
    let f0_with_padding = make_f0_with_padding(f0, PADDING_FRAME_LENGTH);
    let phoneme_with_padding = make_phoneme_with_padding(phoneme, PADDING_FRAME_LENGTH);
    return (length_with_padding, f0_with_padding, phoneme_with_padding);

    fn make_f0_with_padding(
        f0_slice: ndarray::Array1<f32>,
        padding_size: usize,
    ) -> ndarray::Array1<f32> {
        // 音が途切れてしまうのを避けるworkaround処理
        // 改善したらこの関数を削除する
        let padding = ndarray::Array1::<f32>::zeros(padding_size);
        ndarray::concatenate![ndarray::Axis(0), padding, f0_slice, padding]
    }

    fn make_phoneme_with_padding(
        phoneme_slice: ndarray::Array2<f32>,
        padding_size: usize,
    ) -> ndarray::Array2<f32> {
        // 音が途切れてしまうのを避けるworkaround処理
        // 改善したらこの関数を削除する
        let mut padding = ndarray::Array2::<f32>::zeros((padding_size, phoneme_slice.ncols()));
        padding
            .slice_mut(ndarray::s![.., 0])
            .assign(&ndarray::arr0(1.0));
        ndarray::concatenate![ndarray::Axis(0), padding, phoneme_slice, padding]
    }
}
