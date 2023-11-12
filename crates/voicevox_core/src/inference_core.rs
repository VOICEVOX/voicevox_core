use enum_map::enum_map;

use crate::infer::{
    signatures::{
        DecodeInput, DecodeOutput, InferenceKind, PredictDurationInput, PredictDurationOutput,
        PredictIntonationInput, PredictIntonationOutput,
    },
    status::Status,
    InferenceRuntime, InferenceSessionOptions,
};

use super::*;

const PHONEME_LENGTH_MINIMAL: f32 = 0.01;

pub(crate) struct InferenceCore<R: InferenceRuntime> {
    status: Status<R, InferenceKind>,
}

impl<R: InferenceRuntime> InferenceCore<R> {
    pub(crate) fn new(use_gpu: bool, cpu_num_threads: u16) -> Result<Self> {
        if !use_gpu || Self::can_support_gpu_feature()? {
            // 軽いモデルはこちらを使う
            let light_session_options = InferenceSessionOptions::new(cpu_num_threads, false);

            // 重いモデルはこちらを使う
            let heavy_session_options = InferenceSessionOptions::new(cpu_num_threads, use_gpu);

            let status = Status::new(enum_map! {
                InferenceKind::PredictDuration
                | InferenceKind::PredictIntonation => light_session_options,
                InferenceKind::Decode => heavy_session_options,
            });
            Ok(Self { status })
        } else {
            Err(ErrorRepr::GpuSupport.into())
        }
    }

    fn can_support_gpu_feature() -> Result<bool> {
        let supported_devices = SupportedDevices::create()?;

        cfg_if! {
            if #[cfg(feature = "directml")]{
                Ok(*supported_devices.dml())
            } else{
                Ok(*supported_devices.cuda())
            }
        }
    }

    pub async fn load_model(&self, model: &VoiceModel) -> Result<()> {
        let model_bytes = &model.read_inference_models().await?;
        self.status.load_model(model, model_bytes).await
    }

    pub fn unload_model(&self, voice_model_id: &VoiceModelId) -> Result<()> {
        self.status.unload_model(voice_model_id)
    }
    pub fn metas(&self) -> VoiceModelMeta {
        self.status.metas()
    }

    pub fn is_loaded_model(&self, model_id: &VoiceModelId) -> bool {
        self.status.is_loaded_model(model_id)
    }

    pub fn is_model_loaded_by_style_id(&self, style_id: StyleId) -> bool {
        self.status.is_loaded_model_by_style_id(style_id)
    }

    pub async fn predict_duration(
        &self,
        phoneme_vector: &[i64],
        style_id: StyleId,
    ) -> Result<Vec<f32>> {
        if !self.status.validate_speaker_id(style_id) {
            return Err(ErrorRepr::StyleNotFound { style_id }.into());
        }

        let (model_id, model_inner_id) = self.status.ids_for(style_id)?;

        let PredictDurationOutput {
            phoneme_length: output,
        } = self
            .status
            .run_session(
                &model_id,
                PredictDurationInput {
                    phoneme_list: ndarray::arr1(phoneme_vector),
                    speaker_id: ndarray::arr1(&[model_inner_id.raw_id().into()]),
                },
            )
            .await?;
        let mut output = output.into_raw_vec();

        for output_item in output.iter_mut() {
            if *output_item < PHONEME_LENGTH_MINIMAL {
                *output_item = PHONEME_LENGTH_MINIMAL;
            }
        }

        Ok(output)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn predict_intonation(
        &self,
        length: usize,
        vowel_phoneme_vector: &[i64],
        consonant_phoneme_vector: &[i64],
        start_accent_vector: &[i64],
        end_accent_vector: &[i64],
        start_accent_phrase_vector: &[i64],
        end_accent_phrase_vector: &[i64],
        style_id: StyleId,
    ) -> Result<Vec<f32>> {
        if !self.status.validate_speaker_id(style_id) {
            return Err(ErrorRepr::StyleNotFound { style_id }.into());
        }

        let (model_id, model_inner_id) = self.status.ids_for(style_id)?;

        let PredictIntonationOutput { f0_list: output } = self
            .status
            .run_session(
                &model_id,
                PredictIntonationInput {
                    length: ndarray::arr0(length as i64),
                    vowel_phoneme_list: ndarray::arr1(vowel_phoneme_vector),
                    consonant_phoneme_list: ndarray::arr1(consonant_phoneme_vector),
                    start_accent_list: ndarray::arr1(start_accent_vector),
                    end_accent_list: ndarray::arr1(end_accent_vector),
                    start_accent_phrase_list: ndarray::arr1(start_accent_phrase_vector),
                    end_accent_phrase_list: ndarray::arr1(end_accent_phrase_vector),
                    speaker_id: ndarray::arr1(&[model_inner_id.raw_id().into()]),
                },
            )
            .await?;

        Ok(output.into_raw_vec())
    }

    pub async fn decode(
        &self,
        length: usize,
        phoneme_size: usize,
        f0: &[f32],
        phoneme_vector: &[f32],
        style_id: StyleId,
    ) -> Result<Vec<f32>> {
        if !self.status.validate_speaker_id(style_id) {
            return Err(ErrorRepr::StyleNotFound { style_id }.into());
        }

        let (model_id, model_inner_id) = self.status.ids_for(style_id)?;

        // 音が途切れてしまうのを避けるworkaround処理が入っている
        // TODO: 改善したらここのpadding処理を取り除く
        const PADDING_SIZE: f64 = 0.4;
        const DEFAULT_SAMPLING_RATE: f64 = 24000.0;
        let padding_size = ((PADDING_SIZE * DEFAULT_SAMPLING_RATE) / 256.0).round() as usize;
        let start_and_end_padding_size = 2 * padding_size;
        let length_with_padding = length + start_and_end_padding_size;
        let f0_with_padding = Self::make_f0_with_padding(f0, length_with_padding, padding_size);

        let phoneme_with_padding = Self::make_phoneme_with_padding(
            phoneme_vector,
            phoneme_size,
            length_with_padding,
            padding_size,
        );

        let DecodeOutput { wave: output } = self
            .status
            .run_session(
                &model_id,
                DecodeInput {
                    f0: ndarray::arr1(&f0_with_padding)
                        .into_shape([length_with_padding, 1])
                        .unwrap(),
                    phoneme: ndarray::arr1(&phoneme_with_padding)
                        .into_shape([length_with_padding, phoneme_size])
                        .unwrap(),
                    speaker_id: ndarray::arr1(&[model_inner_id.raw_id().into()]),
                },
            )
            .await?;

        Ok(Self::trim_padding_from_output(
            output.into_raw_vec(),
            padding_size,
        ))
    }

    fn make_f0_with_padding(
        f0_slice: &[f32],
        length_with_padding: usize,
        padding_size: usize,
    ) -> Vec<f32> {
        // 音が途切れてしまうのを避けるworkaround処理
        // 改善したらこの関数を削除する
        let mut f0_with_padding = Vec::with_capacity(length_with_padding);
        let padding = vec![0.0; padding_size];
        f0_with_padding.extend_from_slice(&padding);
        f0_with_padding.extend_from_slice(f0_slice);
        f0_with_padding.extend_from_slice(&padding);
        f0_with_padding
    }

    fn make_phoneme_with_padding(
        phoneme_slice: &[f32],
        phoneme_size: usize,
        length_with_padding: usize,
        padding_size: usize,
    ) -> Vec<f32> {
        // 音が途切れてしまうのを避けるworkaround処理
        // 改善したらこの関数を削除する
        let mut padding_phoneme = vec![0.0; phoneme_size];
        padding_phoneme[0] = 1.0;
        let padding_phoneme_len = padding_phoneme.len();
        let padding_phonemes: Vec<f32> = padding_phoneme
            .into_iter()
            .cycle()
            .take(padding_phoneme_len * padding_size)
            .collect();
        let mut phoneme_with_padding = Vec::with_capacity(phoneme_size * length_with_padding);
        phoneme_with_padding.extend_from_slice(&padding_phonemes);
        phoneme_with_padding.extend_from_slice(phoneme_slice);
        phoneme_with_padding.extend_from_slice(&padding_phonemes);

        phoneme_with_padding
    }

    fn trim_padding_from_output(mut output: Vec<f32>, padding_f0_size: usize) -> Vec<f32> {
        // 音が途切れてしまうのを避けるworkaround処理
        // 改善したらこの関数を削除する
        let padding_sampling_size = padding_f0_size * 256;
        output
            .drain(padding_sampling_size..output.len() - padding_sampling_size)
            .collect()
    }
}
