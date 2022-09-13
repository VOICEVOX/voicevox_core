use super::*;
use engine::*;
use once_cell::sync::Lazy;
use onnxruntime::{
    ndarray,
    session::{AnyArray, NdArray},
};
use result_code::VoicevoxResultCode;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::{collections::BTreeMap, path::PathBuf};

use status::*;

pub use status::SupportedDevices;

const PHONEME_LENGTH_MINIMAL: f32 = 0.01;

static SPEAKER_ID_MAP: Lazy<BTreeMap<u32, (usize, u32)>> =
    Lazy::new(|| include!("include_speaker_id_map.rs").into_iter().collect());

pub struct VoicevoxCore {
    synthesis_engine: SynthesisEngine,
    use_gpu: bool,
}

impl VoicevoxCore {
    pub fn new_with_initialize(options: InitializeOptions) -> Result<Self> {
        let mut this = Self::new();
        this.initialize(options)?;
        Ok(this)
    }

    pub fn new_with_mutex() -> Mutex<VoicevoxCore> {
        Mutex::new(Self::new())
    }

    fn new() -> Self {
        Self {
            synthesis_engine: SynthesisEngine::new(
                InferenceCore::new(false, None),
                OpenJtalk::initialize(),
            ),
            use_gpu: false,
        }
    }

    pub fn initialize(&mut self, options: InitializeOptions) -> Result<()> {
        let use_gpu = match options.acceleration_mode {
            AccelerationMode::Auto => {
                let supported_devices = SupportedDevices::get_supported_devices()?;

                cfg_if! {
                    if #[cfg(feature="directml")]{
                        *supported_devices.dml()

                    } else {
                        *supported_devices.cuda()
                    }
                }
            }
            AccelerationMode::Cpu => false,
            AccelerationMode::Gpu => true,
        };
        self.use_gpu = use_gpu;
        self.synthesis_engine.inference_core_mut().initialize(
            use_gpu,
            options.cpu_num_threads,
            options.load_all_models,
        )?;
        if let Some(open_jtalk_dict_dir) = options.open_jtalk_dict_dir {
            self.synthesis_engine
                .load_openjtalk_dict(open_jtalk_dict_dir)?;
        }
        Ok(())
    }

    pub fn is_gpu_mode(&self) -> bool {
        self.use_gpu
    }

    pub fn load_model(&mut self, speaker_id: u32) -> Result<()> {
        self.synthesis_engine
            .inference_core_mut()
            .load_model(speaker_id)
    }

    pub fn is_model_loaded(&self, speaker_id: u32) -> bool {
        self.synthesis_engine
            .inference_core()
            .is_model_loaded(speaker_id)
    }

    pub fn finalize(&mut self) {
        self.synthesis_engine.inference_core_mut().finalize()
    }

    pub const fn get_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    pub fn metas() -> &'static [Meta] {
        &METAS
    }

    pub fn supported_devices() -> &'static SupportedDevices {
        &SUPPORTED_DEVICES
    }

    pub fn predict_duration(
        &mut self,
        phoneme_vector: &[i64],
        speaker_id: u32,
    ) -> Result<Vec<f32>> {
        self.synthesis_engine
            .inference_core_mut()
            .predict_duration(phoneme_vector, speaker_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn predict_intonation(
        &mut self,
        length: usize,
        vowel_phoneme_vector: &[i64],
        consonant_phoneme_vector: &[i64],
        start_accent_vector: &[i64],
        end_accent_vector: &[i64],
        start_accent_phrase_vector: &[i64],
        end_accent_phrase_vector: &[i64],
        speaker_id: u32,
    ) -> Result<Vec<f32>> {
        self.synthesis_engine
            .inference_core_mut()
            .predict_intonation(
                length,
                vowel_phoneme_vector,
                consonant_phoneme_vector,
                start_accent_vector,
                end_accent_vector,
                start_accent_phrase_vector,
                end_accent_phrase_vector,
                speaker_id,
            )
    }

    pub fn decode(
        &mut self,
        length: usize,
        phoneme_size: usize,
        f0: &[f32],
        phoneme_vector: &[f32],
        speaker_id: u32,
    ) -> Result<Vec<f32>> {
        self.synthesis_engine.inference_core_mut().decode(
            length,
            phoneme_size,
            f0,
            phoneme_vector,
            speaker_id,
        )
    }

    pub fn audio_query(
        &mut self,
        text: &str,
        speaker_id: u32,
        options: AudioQueryOptions,
    ) -> Result<AudioQueryModel> {
        if !self.synthesis_engine.is_openjtalk_dict_loaded() {
            return Err(Error::NotLoadedOpenjtalkDict);
        }
        let accent_phrases = if options.kana {
            parse_kana(text)?
        } else {
            self.synthesis_engine
                .create_accent_phrases(text, speaker_id)?
        };

        let kana = create_kana(&accent_phrases);

        Ok(AudioQueryModel::new(
            accent_phrases,
            1.,
            0.,
            1.,
            1.,
            0.1,
            0.1,
            SynthesisEngine::DEFAULT_SAMPLING_RATE,
            false,
            kana,
        ))
    }

    pub fn synthesis(
        &mut self,
        audio_query: &AudioQueryModel,
        speaker_id: u32,
        options: SynthesisOptions,
    ) -> Result<Vec<u8>> {
        self.synthesis_engine.synthesis_wave_format(
            audio_query,
            speaker_id,
            options.enable_interrogative_upspeak,
        )
    }

    pub fn tts(&mut self, text: &str, speaker_id: u32, options: TtsOptions) -> Result<Vec<u8>> {
        let audio_query = &self.audio_query(text, speaker_id, AudioQueryOptions::from(&options))?;
        self.synthesis(audio_query, speaker_id, SynthesisOptions::from(&options))
    }
}

#[derive(Default)]
pub struct AudioQueryOptions {
    pub kana: bool,
}

impl From<&TtsOptions> for AudioQueryOptions {
    fn from(options: &TtsOptions) -> Self {
        Self { kana: options.kana }
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub enum AccelerationMode {
    #[default]
    Auto,
    Cpu,
    Gpu,
}

#[derive(Default)]
pub struct InitializeOptions {
    pub acceleration_mode: AccelerationMode,
    pub cpu_num_threads: u16,
    pub load_all_models: bool,
    pub open_jtalk_dict_dir: Option<PathBuf>,
}

pub struct SynthesisOptions {
    pub enable_interrogative_upspeak: bool,
}

impl From<&TtsOptions> for SynthesisOptions {
    fn from(options: &TtsOptions) -> Self {
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

pub struct TtsOptions {
    pub kana: bool,
    pub enable_interrogative_upspeak: bool,
}

impl Default for TtsOptions {
    fn default() -> Self {
        Self {
            enable_interrogative_upspeak: true,
            kana: Default::default(),
        }
    }
}

#[derive(new)]
pub struct InferenceCore {
    initialized: bool,
    status_option: Option<Status>,
}

impl InferenceCore {
    pub fn initialize(
        &mut self,
        use_gpu: bool,
        cpu_num_threads: u16,
        load_all_models: bool,
    ) -> Result<()> {
        self.initialized = false;
        if !use_gpu || self.can_support_gpu_feature()? {
            let mut status = Status::new(use_gpu, cpu_num_threads);

            status.load_metas()?;

            if load_all_models {
                for model_index in 0..Status::MODELS_COUNT {
                    status.load_model(model_index)?;
                }
            }

            self.status_option = Some(status);
            self.initialized = true;
            Ok(())
        } else {
            Err(Error::GpuSupport)
        }
    }
    fn can_support_gpu_feature(&self) -> Result<bool> {
        let supported_devices = SupportedDevices::get_supported_devices()?;

        cfg_if! {
            if #[cfg(feature = "directml")]{
                Ok(*supported_devices.dml())
            } else{
                Ok(*supported_devices.cuda())
            }
        }
    }
    pub fn load_model(&mut self, speaker_id: u32) -> Result<()> {
        if self.initialized {
            let status = self
                .status_option
                .as_mut()
                .ok_or(Error::UninitializedStatus)?;
            if let Some((model_index, _)) = get_model_index_and_speaker_id(speaker_id) {
                status.load_model(model_index)
            } else {
                Err(Error::InvalidSpeakerId { speaker_id })
            }
        } else {
            Err(Error::UninitializedStatus)
        }
    }
    pub fn is_model_loaded(&self, speaker_id: u32) -> bool {
        if let Some(status) = self.status_option.as_ref() {
            if let Some((model_index, _)) = get_model_index_and_speaker_id(speaker_id) {
                status.is_model_loaded(model_index)
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn finalize(&mut self) {
        self.initialized = false;
        self.status_option = None;
    }

    pub fn predict_duration(
        &mut self,
        phoneme_vector: &[i64],
        speaker_id: u32,
    ) -> Result<Vec<f32>> {
        if !self.initialized {
            return Err(Error::UninitializedStatus);
        }

        let status = self
            .status_option
            .as_mut()
            .ok_or(Error::UninitializedStatus)?;

        if !status.validate_speaker_id(speaker_id) {
            return Err(Error::InvalidSpeakerId { speaker_id });
        }

        let (model_index, speaker_id) =
            if let Some((model_index, speaker_id)) = get_model_index_and_speaker_id(speaker_id) {
                (model_index, speaker_id)
            } else {
                return Err(Error::InvalidSpeakerId { speaker_id });
            };

        if model_index >= Status::MODELS_COUNT {
            return Err(Error::InvalidModelIndex { model_index });
        }

        let mut phoneme_vector_array = NdArray::new(ndarray::arr1(phoneme_vector));
        let mut speaker_id_array = NdArray::new(ndarray::arr1(&[speaker_id as i64]));

        let input_tensors: Vec<&mut dyn AnyArray> =
            vec![&mut phoneme_vector_array, &mut speaker_id_array];

        let mut output = status.predict_duration_session_run(model_index, input_tensors)?;

        for output_item in output.iter_mut() {
            if *output_item < PHONEME_LENGTH_MINIMAL {
                *output_item = PHONEME_LENGTH_MINIMAL;
            }
        }

        Ok(output)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn predict_intonation(
        &mut self,
        length: usize,
        vowel_phoneme_vector: &[i64],
        consonant_phoneme_vector: &[i64],
        start_accent_vector: &[i64],
        end_accent_vector: &[i64],
        start_accent_phrase_vector: &[i64],
        end_accent_phrase_vector: &[i64],
        speaker_id: u32,
    ) -> Result<Vec<f32>> {
        if !self.initialized {
            return Err(Error::UninitializedStatus);
        }

        let status = self
            .status_option
            .as_mut()
            .ok_or(Error::UninitializedStatus)?;

        if !status.validate_speaker_id(speaker_id) {
            return Err(Error::InvalidSpeakerId { speaker_id });
        }

        let (model_index, speaker_id) =
            if let Some((model_index, speaker_id)) = get_model_index_and_speaker_id(speaker_id) {
                (model_index, speaker_id)
            } else {
                return Err(Error::InvalidSpeakerId { speaker_id });
            };

        if model_index >= Status::MODELS_COUNT {
            return Err(Error::InvalidModelIndex { model_index });
        }

        let mut length_array = NdArray::new(ndarray::arr0(length as i64));
        let mut vowel_phoneme_vector_array = NdArray::new(ndarray::arr1(vowel_phoneme_vector));
        let mut consonant_phoneme_vector_array =
            NdArray::new(ndarray::arr1(consonant_phoneme_vector));
        let mut start_accent_vector_array = NdArray::new(ndarray::arr1(start_accent_vector));
        let mut end_accent_vector_array = NdArray::new(ndarray::arr1(end_accent_vector));
        let mut start_accent_phrase_vector_array =
            NdArray::new(ndarray::arr1(start_accent_phrase_vector));
        let mut end_accent_phrase_vector_array =
            NdArray::new(ndarray::arr1(end_accent_phrase_vector));
        let mut speaker_id_array = NdArray::new(ndarray::arr1(&[speaker_id as i64]));

        let input_tensors: Vec<&mut dyn AnyArray> = vec![
            &mut length_array,
            &mut vowel_phoneme_vector_array,
            &mut consonant_phoneme_vector_array,
            &mut start_accent_vector_array,
            &mut end_accent_vector_array,
            &mut start_accent_phrase_vector_array,
            &mut end_accent_phrase_vector_array,
            &mut speaker_id_array,
        ];

        status.predict_intonation_session_run(model_index, input_tensors)
    }

    pub fn decode(
        &mut self,
        length: usize,
        phoneme_size: usize,
        f0: &[f32],
        phoneme_vector: &[f32],
        speaker_id: u32,
    ) -> Result<Vec<f32>> {
        if !self.initialized {
            return Err(Error::UninitializedStatus);
        }

        let status = self
            .status_option
            .as_mut()
            .ok_or(Error::UninitializedStatus)?;

        if !status.validate_speaker_id(speaker_id) {
            return Err(Error::InvalidSpeakerId { speaker_id });
        }

        let (model_index, speaker_id) =
            if let Some((model_index, speaker_id)) = get_model_index_and_speaker_id(speaker_id) {
                (model_index, speaker_id)
            } else {
                return Err(Error::InvalidSpeakerId { speaker_id });
            };

        if model_index >= Status::MODELS_COUNT {
            return Err(Error::InvalidModelIndex { model_index });
        }

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

        let mut f0_array = NdArray::new(
            ndarray::arr1(&f0_with_padding)
                .into_shape([length_with_padding, 1])
                .unwrap(),
        );
        let mut phoneme_array = NdArray::new(
            ndarray::arr1(&phoneme_with_padding)
                .into_shape([length_with_padding, phoneme_size])
                .unwrap(),
        );
        let mut speaker_id_array = NdArray::new(ndarray::arr1(&[speaker_id as i64]));

        let input_tensors: Vec<&mut dyn AnyArray> =
            vec![&mut f0_array, &mut phoneme_array, &mut speaker_id_array];

        status
            .decode_session_run(model_index, input_tensors)
            .map(|output| Self::trim_padding_from_output(output, padding_size))
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

#[derive(Getters, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Style {
    name: String,
    id: u32,
}

#[derive(Getters, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Meta {
    name: String,
    styles: Vec<Style>,
    speaker_uuid: String,
    version: String,
}

static METAS: Lazy<Vec<Meta>> = Lazy::new(|| serde_json::from_str(Status::METAS_STR).unwrap());

static SUPPORTED_DEVICES: Lazy<SupportedDevices> =
    Lazy::new(|| SupportedDevices::get_supported_devices().unwrap());

fn get_model_index_and_speaker_id(speaker_id: u32) -> Option<(usize, u32)> {
    SPEAKER_ID_MAP.get(&speaker_id).copied()
}

pub const fn error_result_to_message(result_code: VoicevoxResultCode) -> &'static str {
    // C APIのため、messageには必ず末尾にNULL文字を追加する
    use VoicevoxResultCode::*;
    match result_code {
        VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR => {
            "voicevox_load_openjtalk_dict() を初めに呼んでください\0"
        }
        VOICEVOX_RESULT_LOAD_MODEL_ERROR => {
            "modelデータ読み込み中にOnnxruntimeエラーが発生しました\0"
        }
        VOICEVOX_RESULT_LOAD_METAS_ERROR => "メタデータ読み込みに失敗しました\0",

        VOICEVOX_RESULT_GPU_SUPPORT_ERROR => "GPU機能をサポートすることができません\0",
        VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR => {
            "サポートされているデバイス情報取得中にエラーが発生しました\0"
        }

        VOICEVOX_RESULT_OK => "エラーが発生しませんでした\0",
        VOICEVOX_RESULT_UNINITIALIZED_STATUS_ERROR => "Statusが初期化されていません\0",
        VOICEVOX_RESULT_INVALID_SPEAKER_ID_ERROR => "無効なspeaker_idです\0",
        VOICEVOX_RESULT_INVALID_MODEL_INDEX_ERROR => "無効なmodel_indexです\0",
        VOICEVOX_RESULT_INFERENCE_ERROR => "推論に失敗しました\0",
        VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR => {
            "入力テキストからのフルコンテキストラベル抽出に失敗しました\0"
        }
        VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR => "入力テキストが無効なUTF-8データでした\0",
        VOICEVOX_RESULT_PARSE_KANA_ERROR => {
            "入力テキストをAquesTalkライクな読み仮名としてパースすることに失敗しました\0"
        }
        VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR => "無効なaudio_queryです\0",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[rstest]
    fn finalize_works() {
        let internal = VoicevoxCore::new_with_mutex();
        let result = internal
            .lock()
            .unwrap()
            .initialize(InitializeOptions::default());
        assert_eq!(Ok(()), result);
        internal.lock().unwrap().finalize();
        assert_eq!(
            false,
            internal
                .lock()
                .unwrap()
                .synthesis_engine
                .inference_core()
                .initialized
        );
        assert_eq!(
            true,
            internal
                .lock()
                .unwrap()
                .synthesis_engine
                .inference_core()
                .status_option
                .is_none()
        );
    }

    #[rstest]
    #[case(0, Err(Error::UninitializedStatus), Ok(()))]
    #[case(1, Err(Error::UninitializedStatus), Ok(()))]
    #[case(999, Err(Error::UninitializedStatus), Err(Error::InvalidSpeakerId{speaker_id:999}))]
    fn load_model_works(
        #[case] speaker_id: u32,
        #[case] expected_result_at_uninitialized: Result<()>,
        #[case] expected_result_at_initialized: Result<()>,
    ) {
        let internal = VoicevoxCore::new_with_mutex();
        let result = internal.lock().unwrap().load_model(speaker_id);
        assert_eq!(expected_result_at_uninitialized, result);

        internal
            .lock()
            .unwrap()
            .initialize(InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            })
            .unwrap();
        let result = internal.lock().unwrap().load_model(speaker_id);
        assert_eq!(
            expected_result_at_initialized, result,
            "got load_model result"
        );
    }

    #[rstest]
    fn is_use_gpu_works() {
        let internal = VoicevoxCore::new_with_mutex();
        assert_eq!(false, internal.lock().unwrap().is_gpu_mode());
        internal
            .lock()
            .unwrap()
            .initialize(InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(false, internal.lock().unwrap().is_gpu_mode());
    }

    #[rstest]
    #[case(0, true)]
    #[case(1, true)]
    #[case(999, false)]
    fn is_model_loaded_works(#[case] speaker_id: u32, #[case] expected: bool) {
        let internal = VoicevoxCore::new_with_mutex();
        assert!(
            !internal.lock().unwrap().is_model_loaded(speaker_id),
            "expected is_model_loaded to return false, but got true",
        );

        internal
            .lock()
            .unwrap()
            .initialize(InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            })
            .unwrap();
        assert!(
            !internal.lock().unwrap().is_model_loaded(speaker_id),
            "expected is_model_loaded to return false, but got true",
        );

        internal
            .lock()
            .unwrap()
            .load_model(speaker_id)
            .unwrap_or(());
        assert_eq!(
            internal.lock().unwrap().is_model_loaded(speaker_id),
            expected,
            "expected is_model_loaded return value against speaker_id `{}` is `{}`, but got `{}`",
            speaker_id,
            expected,
            !expected
        );
    }

    #[rstest]
    #[case(0, Some((0,0)))]
    #[case(1, Some((0,1)))]
    #[case(999, None)]
    fn get_model_index_and_speaker_id_works(
        #[case] speaker_id: u32,
        #[case] expected: Option<(usize, u32)>,
    ) {
        let actual = get_model_index_and_speaker_id(speaker_id);
        assert_eq!(expected, actual);
    }

    #[rstest]
    fn predict_duration_works() {
        let internal = VoicevoxCore::new_with_mutex();
        internal
            .lock()
            .unwrap()
            .initialize(InitializeOptions {
                load_all_models: true,
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            })
            .unwrap();

        // 「こんにちは、音声合成の世界へようこそ」という文章を変換して得た phoneme_vector
        let phoneme_vector = [
            0, 23, 30, 4, 28, 21, 10, 21, 42, 7, 0, 30, 4, 35, 14, 14, 16, 30, 30, 35, 14, 14, 28,
            30, 35, 14, 23, 7, 21, 14, 43, 30, 30, 23, 30, 35, 30, 0,
        ];

        let result = internal
            .lock()
            .unwrap()
            .predict_duration(&phoneme_vector, 0);

        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap().len(), phoneme_vector.len());
    }

    #[rstest]
    fn predict_intonation_works() {
        let internal = VoicevoxCore::new_with_mutex();
        internal
            .lock()
            .unwrap()
            .initialize(InitializeOptions {
                load_all_models: true,
                acceleration_mode: AccelerationMode::Cpu,
                ..Default::default()
            })
            .unwrap();

        // 「テスト」という文章に対応する入力
        let vowel_phoneme_vector = [0, 14, 6, 30, 0];
        let consonant_phoneme_vector = [-1, 37, 35, 37, -1];
        let start_accent_vector = [0, 1, 0, 0, 0];
        let end_accent_vector = [0, 1, 0, 0, 0];
        let start_accent_phrase_vector = [0, 1, 0, 0, 0];
        let end_accent_phrase_vector = [0, 0, 0, 1, 0];

        let result = internal.lock().unwrap().predict_intonation(
            vowel_phoneme_vector.len(),
            &vowel_phoneme_vector,
            &consonant_phoneme_vector,
            &start_accent_vector,
            &end_accent_vector,
            &start_accent_phrase_vector,
            &end_accent_phrase_vector,
            0,
        );

        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap().len(), vowel_phoneme_vector.len());
    }

    #[rstest]
    fn decode_works() {
        let internal = VoicevoxCore::new_with_mutex();
        internal
            .lock()
            .unwrap()
            .initialize(InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                load_all_models: true,
                ..Default::default()
            })
            .unwrap();

        // 「テスト」という文章に対応する入力
        const F0_LENGTH: usize = 69;
        let mut f0 = [0.; F0_LENGTH];
        f0[9..24].fill(5.905218);
        f0[37..60].fill(5.565851);

        const PHONEME_SIZE: usize = 45;
        let mut phoneme = [0.; PHONEME_SIZE * F0_LENGTH];
        let mut set_one = |index, range| {
            for i in range {
                phoneme[i * PHONEME_SIZE + index] = 1.;
            }
        };
        set_one(0, 0..9);
        set_one(37, 9..13);
        set_one(14, 13..24);
        set_one(35, 24..30);
        set_one(6, 30..37);
        set_one(37, 37..45);
        set_one(30, 45..60);
        set_one(0, 60..69);

        let result = internal
            .lock()
            .unwrap()
            .decode(F0_LENGTH, PHONEME_SIZE, &f0, &phoneme, 0);

        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap().len(), F0_LENGTH * 256);
    }

    #[rstest]
    #[async_std::test]
    async fn audio_query_works() {
        let open_jtalk_dic_dir = download_open_jtalk_dict_if_no_exists().await;

        let core = VoicevoxCore::new_with_mutex();
        core.lock()
            .unwrap()
            .initialize(InitializeOptions {
                acceleration_mode: AccelerationMode::Cpu,
                load_all_models: true,
                open_jtalk_dict_dir: Some(open_jtalk_dic_dir),
                ..Default::default()
            })
            .unwrap();

        let query = core
            .lock()
            .unwrap()
            .audio_query("これはテストです", 0, Default::default())
            .unwrap();

        assert_eq!(query.accent_phrases().len(), 2);

        assert_eq!(query.accent_phrases()[0].moras().len(), 3);
        for (i, (text, consonant, vowel)) in [("コ", "k", "o"), ("レ", "r", "e"), ("ワ", "w", "a")]
            .iter()
            .enumerate()
        {
            let mora = query.accent_phrases()[0].moras().get(i).unwrap();
            assert_eq!(mora.text(), text);
            assert_eq!(mora.consonant(), &Some(consonant.to_string()));
            assert_eq!(mora.vowel(), vowel);
        }
        assert_eq!(query.accent_phrases()[0].accent(), &3);

        assert_eq!(query.accent_phrases()[1].moras().len(), 5);
        for (i, (text, consonant, vowel)) in [
            ("テ", "t", "e"),
            ("ス", "s", "U"),
            ("ト", "t", "o"),
            ("デ", "d", "e"),
            ("ス", "s", "U"),
        ]
        .iter()
        .enumerate()
        {
            let mora = query.accent_phrases()[1].moras().get(i).unwrap();
            assert_eq!(mora.text(), text);
            assert_eq!(mora.consonant(), &Some(consonant.to_string()));
            assert_eq!(mora.vowel(), vowel);
        }
        assert_eq!(query.accent_phrases()[1].accent(), &1);
        assert_eq!(query.kana(), "コレワ'/テ'_ストデ_ス");
    }

    #[rstest]
    fn get_version_works() {
        assert_eq!("0.0.0", VoicevoxCore::get_version());
    }
}
