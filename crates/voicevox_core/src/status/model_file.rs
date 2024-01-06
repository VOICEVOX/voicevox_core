use super::{DecryptModelError, TalkModelFileNames, SingStyleModelFileNames, SourceFilterModelFileNames};

pub(super) fn decrypt(content: &[u8]) -> std::result::Result<Vec<u8>, DecryptModelError> {
    Ok(content.to_owned())
}

pub(super) const TALK_SPEAKER_ID_MAP: &[(u32, (usize, u32))] = &[(0, (0, 0)), (1, (0, 1))];

pub(super) const TALK_MODEL_FILE_NAMES: &[TalkModelFileNames] = &[
    TalkModelFileNames {
        predict_duration_model: "predict_duration-0.onnx",
        predict_intonation_model: "predict_intonation-0.onnx",
        decode_model: "decode-0.onnx",
    },
];

// TODO: 変更する
pub(super) const SING_STYLE_SPEAKER_ID_MAP: &[(u32, (usize, u32))] = &[(0, (0, 0)), (1, (0, 1))];

pub(super) const SING_STYLE_MODEL_FILE_NAMES: &[SingStyleModelFileNames] = &[
    SingStyleModelFileNames {
        predict_sing_consonant_length_model: "predict_duration-1.onnx",
        predict_sing_f0_model: "predict_intonation-1.onnx",
        predict_sing_volume_model: "predict_intonation-1.onnx",
    },
];

pub(super) const SOURCE_FILTER_SPEAKER_ID_MAP: &[(u32, (usize, u32))] = &[(0, (0, 0)), (1, (0, 1))];

pub(super) const SOURCE_FILTER_MODEL_FILE_NAMES: &[SourceFilterModelFileNames] = &[
    SourceFilterModelFileNames {
        source_filter_decode_model: "decode-1.onnx",
    },
];

