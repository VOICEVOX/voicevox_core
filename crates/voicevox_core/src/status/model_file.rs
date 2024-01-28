use super::{
    DecryptModelError, SfDecodeModelFileNames, SingTeacherModelFileNames, TalkModelFileNames,
};

pub(super) fn decrypt(content: &[u8]) -> std::result::Result<Vec<u8>, DecryptModelError> {
    Ok(content.to_owned())
}

pub(super) const TALK_SPEAKER_ID_MAP: &[(u32, (usize, u32))] = &[(0, (0, 0)), (1, (0, 1))];

pub(super) const TALK_MODEL_FILE_NAMES: &[TalkModelFileNames] = &[TalkModelFileNames {
    predict_duration_model: "predict_duration-0.onnx",
    predict_intonation_model: "predict_intonation-0.onnx",
    decode_model: "decode-0.onnx",
}];

pub(super) const SING_TEACHER_SPEAKER_ID_MAP: &[(u32, (usize, u32))] = &[(6000, (0, 0))];

pub(super) const SING_TEACHER_MODEL_FILE_NAMES: &[SingTeacherModelFileNames] =
    &[SingTeacherModelFileNames {
        predict_sing_consonant_length_model: "predict_sing_consonant_length-0.onnx",
        predict_sing_f0_model: "predict_sing_f0-0.onnx",
        predict_sing_volume_model: "predict_sing_volume-0.onnx",
    }];

pub(super) const SF_DECODE_SPEAKER_ID_MAP: &[(u32, (usize, u32))] = &[(3000, (0, 0))];

pub(super) const SF_DECODE_MODEL_FILE_NAMES: &[SfDecodeModelFileNames] =
    &[SfDecodeModelFileNames {
        sf_decode_model: "sf_decode-0.onnx",
    }];
