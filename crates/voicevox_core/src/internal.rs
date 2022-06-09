use super::*;
use c_export::VoicevoxResultCode;
use once_cell::sync::Lazy;
use std::collections::BTreeMap;
use std::ffi::CStr;
use std::os::raw::c_int;
use std::sync::Mutex;

use status::*;
use std::ffi::CString;

static SPEAKER_ID_MAP: Lazy<BTreeMap<usize, (usize, usize)>> = Lazy::new(BTreeMap::new);

pub struct Internal {
    initialized: bool,
    status_option: Option<Status>,
}

impl Internal {
    pub fn new_with_mutex() -> Mutex<Internal> {
        Mutex::new(Internal {
            initialized: false,
            status_option: None,
        })
    }
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
                // TODO: ここにGPUメモリを確保させる処理を実装する
                // https://github.com/VOICEVOX/voicevox_core/blob/main/core/src/core.cpp#L210-L219
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
            let (model_index, _) = get_model_index_and_speaker_id(speaker_id);
            status.load_model(model_index)
        } else {
            Err(Error::UninitializedStatus)
        }
    }
    pub fn is_model_loaded(&self, speaker_id: usize) -> bool {
        if let Some(status) = self.status_option.as_ref() {
            let (model_index, _) = get_model_index_and_speaker_id(speaker_id);
            status.is_model_loaded(model_index)
        } else {
            false
        }
    }
    pub fn finalize(&mut self) {
        unimplemented!()
    }
    pub fn metas(&self) -> &'static CStr {
        &METAS_CSTRING
    }
    pub fn supported_devices(&self) -> &'static CStr {
        &SUPPORTED_DEVICES_CSTRING
    }
    //TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
    #[allow(unused_variables)]
    pub fn yukarin_s_forward(
        &mut self,
        length: i64,
        phoneme_list: *const i64,
        speaker_id: &i64,
        output: *mut f32,
    ) -> Result<()> {
        unimplemented!()
    }

    //TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
    pub fn yukarin_sa_forward(
        &mut self,
        length: i64,
        vowel_phoneme_list: *const i64,
        consonant_phoneme_list: *const i64,
        start_accent_list: *const i64,
        end_accent_list: *const i64,
        start_accent_phrase_list: *const i64,
        end_accent_phrase_list: *const i64,
        speaker_id: *const i64,
        output: *mut f32,
    ) -> Result<()> {
        unimplemented!()
    }

    //TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
    #[allow(unused_variables)]
    pub fn decode_forward(
        &mut self,
        length: i64,
        phoneme_size: i64,
        f0: *const f32,
        phoneme: *const f32,
        speaker_id: *const i64,
        output: *mut f32,
    ) -> Result<()> {
        unimplemented!()
    }

    //TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
    #[allow(unused_variables)]
    pub fn voicevox_load_openjtalk_dict(&mut self, dict_path: &CStr) -> Result<()> {
        unimplemented!()
    }

    //TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
    #[allow(unused_variables)]
    pub fn voicevox_tts(
        &self,
        text: &CStr,
        speaker_id: i64,
        output_binary_size: *mut c_int,
        output_wav: *const *mut u8,
    ) -> Result<()> {
        unimplemented!()
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

    //TODO:仮実装がlinterエラーにならないようにするための属性なのでこの関数を正式に実装する際にallow(unused_variables)を取り除くこと
    #[allow(unused_variables)]
    pub fn voicevox_wav_free(&self, wav: *mut u8) -> Result<()> {
        unimplemented!()
    }
}

static METAS_CSTRING: Lazy<CString> = Lazy::new(|| CString::new(Status::METAS_STR).unwrap());

static SUPPORTED_DEVICES_CSTRING: Lazy<CString> = Lazy::new(|| {
    CString::new(
        serde_json::to_string(&SupportedDevices::get_supported_devices().unwrap()).unwrap(),
    )
    .unwrap()
});

fn get_model_index_and_speaker_id(speaker_id: usize) -> (usize, usize) {
    *SPEAKER_ID_MAP.get(&speaker_id).unwrap_or(&(0, speaker_id))
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[fixture]
    #[once]
    fn internal_uninitialized() -> Internal {
        Internal {
            initialized: false,
            status_option: None,
        }
    }

    #[fixture]
    #[once]
    fn internal_initialized_but_model_unloaded() -> Internal {
        let mut internal = Internal {
            initialized: false,
            status_option: None,
        };
        internal.initialize(false, 0, false).unwrap();
        internal
    }

    #[fixture]
    #[once]
    fn internal_initialized_and_model_loaded() -> Internal {
        let mut internal = Internal {
            initialized: false,
            status_option: None,
        };
        internal.initialize(false, 0, true).unwrap();
        internal
    }

    #[rstest]
    #[case(0, false, true)]
    #[case(3, false, true)]
    fn load_model_works(
        #[case] speaker_id: usize,
        #[case] expected_at_uninitialized: bool,
        #[case] expected_at_initialized: bool,
    ) {
        let internal = Internal::new_with_mutex();
        let result = internal.lock().unwrap().load_model(speaker_id);
        assert_eq!(
            expected_at_uninitialized,
            result.is_ok(),
            "expected load_model to be failed, but succeed wrongly",
        );

        internal
            .lock()
            .unwrap()
            .initialize(false, 0, false)
            .unwrap();
        let result = internal.lock().unwrap().load_model(speaker_id);
        assert_eq!(
            expected_at_initialized,
            result.is_ok(),
            "got load_model result: {:?}",
            result
        );
    }

    #[rstest]
    #[case(0, true)]
    #[case(1, true)]
    #[case(3, true)]
    fn is_model_loaded_works(
        #[case] speaker_id: usize,
        internal_uninitialized: &Internal,
        internal_initialized_but_model_unloaded: &Internal,
        internal_initialized_and_model_loaded: &Internal,
        #[case] expected_at_model_loaded: bool,
    ) {
        assert!(
            !internal_uninitialized.is_model_loaded(speaker_id),
            "expected is_model_loaded to return false, but got true",
        );

        assert!(
            !internal_initialized_but_model_unloaded.is_model_loaded(speaker_id),
            "expected is_model_loaded to return false, but got true",
        );

        assert_eq!(
            internal_initialized_and_model_loaded.is_model_loaded(speaker_id),
            expected_at_model_loaded,
            "expected is_model_loaded return value against speaker_id `{}` is `{}`, but got `{}`",
            speaker_id,
            expected_at_model_loaded,
            !expected_at_model_loaded
        );
    }

    #[rstest]
    fn supported_devices_works(internal_uninitialized: &Internal) {
        let cstr_result = internal_uninitialized.supported_devices();
        assert!(cstr_result.to_str().is_ok(), "{:?}", cstr_result);

        let json_result: std::result::Result<SupportedDevices, _> =
            serde_json::from_str(cstr_result.to_str().unwrap());
        assert!(json_result.is_ok(), "{:?}", json_result);
    }

    #[rstest]
    #[case(0,(0,0))]
    #[case(1,(0,1))]
    #[case(3,(0,3))]
    fn get_model_index_and_speaker_id_works(
        #[case] speaker_id: usize,
        #[case] expected: (usize, usize),
    ) {
        let actual = get_model_index_and_speaker_id(speaker_id);
        assert_eq!(expected, actual);
    }
}
