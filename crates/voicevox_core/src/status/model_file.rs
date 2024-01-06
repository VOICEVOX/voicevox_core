use super::{DecryptModelError, TalkModelFileNames};

pub(super) fn decrypt(content: &[u8]) -> std::result::Result<Vec<u8>, DecryptModelError> {
    Ok(content.to_owned())
}

pub(super) const SPEAKER_ID_MAP: &[(u32, (usize, u32))] =
    &[(0, (0, 0)), (1, (0, 1))];

pub(super) const TALK_MODEL_FILE_NAMES: &[TalkModelFileNames] = &[
    TalkModelFileNames {
        predict_duration_model: "predict_duration-0.onnx",
        predict_intonation_model: "predict_intonation-0.onnx",
        decode_model: "decode-0.onnx",
    },
];
