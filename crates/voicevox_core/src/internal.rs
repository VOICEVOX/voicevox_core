use super::*;
use c_export::VoicevoxResultCode;
use engine::*;
use once_cell::sync::Lazy;
use onnxruntime::{
    ndarray,
    session::{AnyArray, NdArray},
};
use std::collections::BTreeMap;
use std::ffi::CStr;
use std::os::raw::c_int;
use std::sync::Mutex;

use status::*;
use std::ffi::CString;

const PHONEME_LENGTH_MINIMAL: f32 = 0.01;

static SPEAKER_ID_MAP: Lazy<BTreeMap<usize, (usize, usize)>> =
    Lazy::new(|| include!("include_speaker_id_map.rs").into_iter().collect());

pub struct Internal {
    synthesis_engine: SynthesisEngine,
}

impl Internal {
    pub fn new_with_mutex() -> Mutex<Internal> {
        Mutex::new(Internal {
            synthesis_engine: SynthesisEngine::new(InferenceCore::new(false, None)),
        })
    }

    pub fn initialize(
        &mut self,
        use_gpu: bool,
        cpu_num_threads: usize,
        load_all_models: bool,
    ) -> Result<()> {
        self.synthesis_engine.inference_core_mut().initialize(
            use_gpu,
            cpu_num_threads,
            load_all_models,
        )
    }

    pub fn load_model(&mut self, speaker_id: usize) -> Result<()> {
        self.synthesis_engine
            .inference_core_mut()
            .load_model(speaker_id)
    }

    pub fn is_model_loaded(&self, speaker_id: usize) -> bool {
        self.synthesis_engine
            .inference_core()
            .is_model_loaded(speaker_id)
    }

    pub fn finalize(&mut self) {
        self.synthesis_engine.inference_core_mut().finalize()
    }

    pub fn metas(&self) -> &'static CStr {
        &METAS_CSTRING
    }

    pub fn supported_devices(&self) -> &'static CStr {
        &SUPPORTED_DEVICES_CSTRING
    }

    pub fn yukarin_s_forward(
        &mut self,
        phoneme_list: &[i64],
        speaker_id: usize,
    ) -> Result<Vec<f32>> {
        self.synthesis_engine
            .inference_core_mut()
            .yukarin_s_forward(phoneme_list, speaker_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn yukarin_sa_forward(
        &mut self,
        length: i64,
        vowel_phoneme_list: &[i64],
        consonant_phoneme_list: &[i64],
        start_accent_list: &[i64],
        end_accent_list: &[i64],
        start_accent_phrase_list: &[i64],
        end_accent_phrase_list: &[i64],
        speaker_id: usize,
    ) -> Result<Vec<f32>> {
        self.synthesis_engine
            .inference_core_mut()
            .yukarin_sa_forward(
                length,
                vowel_phoneme_list,
                consonant_phoneme_list,
                start_accent_list,
                end_accent_list,
                start_accent_phrase_list,
                end_accent_phrase_list,
                speaker_id,
            )
    }

    pub fn decode_forward(
        &mut self,
        length: usize,
        phoneme_size: usize,
        f0: &[f32],
        phoneme: &[f32],
        speaker_id: usize,
    ) -> Result<Vec<f32>> {
        self.synthesis_engine.inference_core_mut().decode_forward(
            length,
            phoneme_size,
            f0,
            phoneme,
            speaker_id,
        )
    }

    pub fn voicevox_load_openjtalk_dict(&mut self, dict_path: &str) -> Result<()> {
        self.synthesis_engine.load_openjtalk_dict(dict_path)
    }

    pub fn voicevox_tts(&mut self, text: &str, speaker_id: usize) -> Result<Vec<u8>> {
        if !self.synthesis_engine.is_openjtalk_dict_loaded() {
            return Err(Error::NotLoadedOpenjtalkDict);
        }
        let accent_phrases = self
            .synthesis_engine
            .create_accent_phrases(text, speaker_id)?;

        let audio_query = AudioQueryModel::new(
            accent_phrases,
            1.,
            0.,
            1.,
            1.,
            0.1,
            0.1,
            SynthesisEngine::DEFAULT_SAMPLING_RATE,
            false,
            "".into(),
        );

        self.synthesis_engine
            .synthesis_wave_format(&audio_query, speaker_id, true) // TODO: 疑問文化を設定可能にする
    }

    //TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
    #[allow(unused_variables)]
    pub fn voicevox_tts_from_kana(
        &self,
        text: &CStr,
        speaker_id: i64,
        output_binary_size: *mut c_int,
        output_wav: *const *mut u8,
    ) -> Result<()> {
        unimplemented!()
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
        cpu_num_threads: usize,
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
                // 一回走らせて十分なGPUメモリを確保させる
                // TODO: 全MODELに対して行う
                if use_gpu {
                    const LENGTH: usize = 500;
                    const PHONEME_SIZE: usize = 45;
                    let f0 = [0.; LENGTH];
                    let phoneme = [0.; PHONEME_SIZE * LENGTH];
                    let speaker_id = 0;

                    let _ = self.decode_forward(LENGTH, PHONEME_SIZE, &f0, &phoneme, speaker_id)?;
                }
            }

            self.status_option = Some(status);
            self.initialized = true;
            Ok(())
        } else {
            Err(Error::CantGpuSupport)
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
    pub fn load_model(&mut self, speaker_id: usize) -> Result<()> {
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
    pub fn is_model_loaded(&self, speaker_id: usize) -> bool {
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

    pub fn yukarin_s_forward(
        &mut self,
        phoneme_list: &[i64],
        speaker_id: usize,
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

        let mut phoneme_list_array = NdArray::new(ndarray::arr1(phoneme_list));
        let mut speaker_id_array = NdArray::new(ndarray::arr1(&[speaker_id as i64]));

        let input_tensors: Vec<&mut dyn AnyArray> =
            vec![&mut phoneme_list_array, &mut speaker_id_array];

        let mut output = status.yukarin_s_session_run(model_index, input_tensors)?;

        for output_item in output.iter_mut() {
            if *output_item < PHONEME_LENGTH_MINIMAL {
                *output_item = PHONEME_LENGTH_MINIMAL;
            }
        }

        Ok(output)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn yukarin_sa_forward(
        &mut self,
        length: i64,
        vowel_phoneme_list: &[i64],
        consonant_phoneme_list: &[i64],
        start_accent_list: &[i64],
        end_accent_list: &[i64],
        start_accent_phrase_list: &[i64],
        end_accent_phrase_list: &[i64],
        speaker_id: usize,
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

        let mut length_array = NdArray::new(ndarray::arr0(length));
        let mut vowel_phoneme_list_array = NdArray::new(ndarray::arr1(vowel_phoneme_list));
        let mut consonant_phoneme_list_array = NdArray::new(ndarray::arr1(consonant_phoneme_list));
        let mut start_accent_list_array = NdArray::new(ndarray::arr1(start_accent_list));
        let mut end_accent_list_array = NdArray::new(ndarray::arr1(end_accent_list));
        let mut start_accent_phrase_list_array =
            NdArray::new(ndarray::arr1(start_accent_phrase_list));
        let mut end_accent_phrase_list_array = NdArray::new(ndarray::arr1(end_accent_phrase_list));
        let mut speaker_id_array = NdArray::new(ndarray::arr1(&[speaker_id as i64]));

        let input_tensors: Vec<&mut dyn AnyArray> = vec![
            &mut length_array,
            &mut vowel_phoneme_list_array,
            &mut consonant_phoneme_list_array,
            &mut start_accent_list_array,
            &mut end_accent_list_array,
            &mut start_accent_phrase_list_array,
            &mut end_accent_phrase_list_array,
            &mut speaker_id_array,
        ];

        status.yukarin_sa_session_run(model_index, input_tensors)
    }

    pub fn decode_forward(
        &mut self,
        length: usize,
        phoneme_size: usize,
        f0: &[f32],
        phoneme: &[f32],
        speaker_id: usize,
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
            phoneme,
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

static METAS_CSTRING: Lazy<CString> = Lazy::new(|| CString::new(Status::METAS_STR).unwrap());

static SUPPORTED_DEVICES_CSTRING: Lazy<CString> = Lazy::new(|| {
    CString::new(
        serde_json::to_string(&SupportedDevices::get_supported_devices().unwrap()).unwrap(),
    )
    .unwrap()
});

fn get_model_index_and_speaker_id(speaker_id: usize) -> Option<(usize, usize)> {
    SPEAKER_ID_MAP.get(&speaker_id).copied()
}

pub const fn voicevox_error_result_to_message(result_code: VoicevoxResultCode) -> &'static str {
    // C APIのため、messageには必ず末尾にNULL文字を追加する
    use VoicevoxResultCode::*;
    match result_code {
        VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT => {
            "voicevox_load_openjtalk_dict() を初めに呼んでください\0"
        }
        VOICEVOX_RESULT_FAILED_LOAD_MODEL => {
            "modelデータ読み込み中にOnnxruntimeエラーが発生しました\0"
        }
        VOICEVOX_RESULT_FAILED_LOAD_METAS => "メタデータ読み込みに失敗しました\0",

        VOICEVOX_RESULT_CANT_GPU_SUPPORT => "GPU機能をサポートすることができません\0",
        VOICEVOX_RESULT_FAILED_GET_SUPPORTED_DEVICES => {
            "サポートされているデバイス情報取得中にエラーが発生しました\0"
        }

        VOICEVOX_RESULT_SUCCEED => "エラーが発生しませんでした\0",
        VOICEVOX_RESULT_UNINITIALIZED_STATUS => "Statusが初期化されていません\0",
        VOICEVOX_RESULT_INVALID_SPEAKER_ID => "無効なspeaker_idです\0",
        VOICEVOX_RESULT_INVALID_MODEL_INDEX => "無効なmodel_indexです\0",
        VOICEVOX_RESULT_INFERENCE_FAILED => "推論に失敗しました\0",
        VOICEVOX_RESULT_FAILED_EXTRACT_FULL_CONTEXT_LABEL => {
            "入力テキストからのフルコンテキストラベル抽出に失敗しました\0"
        }
        VOICEVOX_RESULT_INVALID_UTF8_INPUT => "入力テキストが無効なUTF-8データでした\0",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[rstest]
    fn finalize_works() {
        let internal = Internal::new_with_mutex();
        let result = internal.lock().unwrap().initialize(false, 0, false);
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
    #[case(3, Err(Error::UninitializedStatus), Err(Error::InvalidSpeakerId{speaker_id:3}))]
    fn load_model_works(
        #[case] speaker_id: usize,
        #[case] expected_result_at_uninitialized: Result<()>,
        #[case] expected_result_at_initialized: Result<()>,
    ) {
        let internal = Internal::new_with_mutex();
        let result = internal.lock().unwrap().load_model(speaker_id);
        assert_eq!(expected_result_at_uninitialized, result);

        internal
            .lock()
            .unwrap()
            .initialize(false, 0, false)
            .unwrap();
        let result = internal.lock().unwrap().load_model(speaker_id);
        assert_eq!(
            expected_result_at_initialized, result,
            "got load_model result"
        );
    }

    #[rstest]
    #[case(0, true)]
    #[case(1, true)]
    #[case(3, false)]
    fn is_model_loaded_works(#[case] speaker_id: usize, #[case] expected: bool) {
        let internal = Internal::new_with_mutex();
        assert!(
            !internal.lock().unwrap().is_model_loaded(speaker_id),
            "expected is_model_loaded to return false, but got true",
        );

        internal
            .lock()
            .unwrap()
            .initialize(false, 0, false)
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
    fn supported_devices_works() {
        let internal = Internal::new_with_mutex();
        let cstr_result = internal.lock().unwrap().supported_devices();
        assert!(cstr_result.to_str().is_ok(), "{:?}", cstr_result);

        let json_result: std::result::Result<SupportedDevices, _> =
            serde_json::from_str(cstr_result.to_str().unwrap());
        assert!(json_result.is_ok(), "{:?}", json_result);
    }

    #[rstest]
    #[case(0, Some((0,0)))]
    #[case(1, Some((0,1)))]
    #[case(3, None)]
    fn get_model_index_and_speaker_id_works(
        #[case] speaker_id: usize,
        #[case] expected: Option<(usize, usize)>,
    ) {
        let actual = get_model_index_and_speaker_id(speaker_id);
        assert_eq!(expected, actual);
    }

    #[rstest]
    fn yukarin_s_forward_works() {
        let internal = Internal::new_with_mutex();
        internal.lock().unwrap().initialize(false, 0, true).unwrap();

        // 「こんにちは、音声合成の世界へようこそ」という文章を変換して得た phoneme_list
        let phoneme_list = [
            0, 23, 30, 4, 28, 21, 10, 21, 42, 7, 0, 30, 4, 35, 14, 14, 16, 30, 30, 35, 14, 14, 28,
            30, 35, 14, 23, 7, 21, 14, 43, 30, 30, 23, 30, 35, 30, 0,
        ];

        let result = internal.lock().unwrap().yukarin_s_forward(&phoneme_list, 0);

        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap().len(), phoneme_list.len());
    }

    #[rstest]
    fn yukarin_sa_forward_works() {
        let internal = Internal::new_with_mutex();
        internal.lock().unwrap().initialize(false, 0, true).unwrap();

        // 「テスト」という文章に対応する入力
        let vowel_phoneme_list = [0, 14, 6, 30, 0];
        let consonant_phoneme_list = [-1, 37, 35, 37, -1];
        let start_accent_list = [0, 1, 0, 0, 0];
        let end_accent_list = [0, 1, 0, 0, 0];
        let start_accent_phrase_list = [0, 1, 0, 0, 0];
        let end_accent_phrase_list = [0, 0, 0, 1, 0];

        let result = internal.lock().unwrap().yukarin_sa_forward(
            vowel_phoneme_list.len() as i64,
            &vowel_phoneme_list,
            &consonant_phoneme_list,
            &start_accent_list,
            &end_accent_list,
            &start_accent_phrase_list,
            &end_accent_phrase_list,
            0,
        );

        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap().len(), vowel_phoneme_list.len());
    }

    #[rstest]
    fn decode_forward_works() {
        let internal = Internal::new_with_mutex();
        internal.lock().unwrap().initialize(false, 0, true).unwrap();

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

        let result =
            internal
                .lock()
                .unwrap()
                .decode_forward(F0_LENGTH, PHONEME_SIZE, &f0, &phoneme, 0);

        assert!(result.is_ok(), "{:?}", result);
        assert_eq!(result.unwrap().len(), F0_LENGTH * 256);
    }

    #[rstest]
    #[async_std::test]
    async fn voicevox_load_openjtalk_dict_works() {
        let internal = Internal::new_with_mutex();
        let open_jtalk_dic_dir = download_open_jtalk_dict_if_no_exists().await;
        let result = internal
            .lock()
            .unwrap()
            .voicevox_load_openjtalk_dict(open_jtalk_dic_dir.to_str().unwrap());
        assert_eq!(result, Ok(()));
    }
}
