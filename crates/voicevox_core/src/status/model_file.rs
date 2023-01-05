use super::{DecryptModelError, ModelFileNames};

pub(super) fn decrypt(content: &[u8]) -> std::result::Result<Vec<u8>, DecryptModelError> {
    Ok(content.to_owned())
}

pub(super) const SPEAKER_ID_MAP: &[(u32, (usize, u32))] =
    &[(0, (0, 0)), (1, (0, 1)), (2, (1, 0)), (3, (1, 1))];

pub(super) const MODEL_FILE_NAMES: &[ModelFileNames] = &[
    ModelFileNames {
        predict_duration_model: "predict_duration-0.onnx",
        predict_intonation_model: "predict_intonation-0.onnx",
        decode_model: "decode-0.onnx",
    },
    ModelFileNames {
        predict_duration_model: "predict_duration-1.onnx",
        predict_intonation_model: "predict_intonation-1.onnx",
        decode_model: "decode-1.onnx",
    },
];
