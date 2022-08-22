use super::*;

pub(crate) fn convert_result<T>(result: Result<T>) -> (Option<T>, VoicevoxResultCode) {
    match result {
        Ok(target) => (Some(target), VoicevoxResultCode::VOICEVOX_RESULT_SUCCEED),
        Err(err) => {
            eprintln!("{}", err);
            dbg!(&err);
            match err {
                Error::NotLoadedOpenjtalkDict => (
                    None,
                    VoicevoxResultCode::VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT,
                ),
                Error::CantGpuSupport => {
                    (None, VoicevoxResultCode::VOICEVOX_RESULT_CANT_GPU_SUPPORT)
                }
                Error::LoadModel(_) => {
                    (None, VoicevoxResultCode::VOICEVOX_RESULT_FAILED_LOAD_MODEL)
                }
                Error::LoadMetas(_) => {
                    (None, VoicevoxResultCode::VOICEVOX_RESULT_FAILED_LOAD_METAS)
                }
                Error::GetSupportedDevices(_) => (
                    None,
                    VoicevoxResultCode::VOICEVOX_RESULT_FAILED_GET_SUPPORTED_DEVICES,
                ),
                Error::UninitializedStatus => (
                    None,
                    VoicevoxResultCode::VOICEVOX_RESULT_UNINITIALIZED_STATUS,
                ),
                Error::InvalidSpeakerId { .. } => {
                    (None, VoicevoxResultCode::VOICEVOX_RESULT_INVALID_SPEAKER_ID)
                }
                Error::InvalidModelIndex { .. } => (
                    None,
                    VoicevoxResultCode::VOICEVOX_RESULT_INVALID_MODEL_INDEX,
                ),
                Error::InferenceFailed => {
                    (None, VoicevoxResultCode::VOICEVOX_RESULT_INFERENCE_FAILED)
                }
                Error::FailedExtractFullContextLabel(_) => (
                    None,
                    VoicevoxResultCode::VOICEVOX_RESULT_FAILED_EXTRACT_FULL_CONTEXT_LABEL,
                ),
                Error::FailedParseKana(_) => {
                    (None, VoicevoxResultCode::VOICEVOX_RESULT_FAILED_PARSE_KANA)
                }
            }
        }
    }
}

pub(crate) fn create_audio_query(
    japanese_or_kana: &CStr,
    speaker_id: usize,
    method: fn(
        &mut Internal,
        &str,
        usize,
        voicevox_core::AudioQueryOptions,
    ) -> Result<AudioQueryModel>,
    options: VoicevoxAudioQueryOptions,
) -> std::result::Result<CString, VoicevoxResultCode> {
    let japanese_or_kana = ensure_utf8(japanese_or_kana)?;

    let (audio_query, result_code) = convert_result(method(
        &mut lock_internal(),
        japanese_or_kana,
        speaker_id,
        options.into(),
    ));
    let audio_query = audio_query.ok_or(result_code)?;
    Ok(CString::new(audio_query_model_to_json(&audio_query)).expect("should not contain '\\0'"))
}

fn audio_query_model_to_json(audio_query_model: &AudioQueryModel) -> String {
    serde_json::to_string(audio_query_model).expect("should be always valid")
}

pub(crate) unsafe fn write_json_to_ptr(output_ptr: *mut *mut c_char, json: &CStr) {
    let n = json.to_bytes_with_nul().len();
    let json_heap = libc::malloc(n);
    libc::memcpy(json_heap, json.as_ptr() as *const c_void, n);
    output_ptr.write(json_heap as *mut c_char);
}

pub(crate) unsafe fn write_wav_to_ptr(
    output_wav_ptr: *mut *mut u8,
    output_size_ptr: *mut c_int,
    data: &[u8],
) {
    output_size_ptr.write(data.len() as c_int);
    let wav_heap = libc::malloc(data.len());
    libc::memcpy(wav_heap, data.as_ptr() as *const c_void, data.len());
    output_wav_ptr.write(wav_heap as *mut u8);
}

pub(crate) fn ensure_utf8(s: &CStr) -> std::result::Result<&str, VoicevoxResultCode> {
    s.to_str()
        .map_err(|_| VoicevoxResultCode::VOICEVOX_RESULT_INVALID_UTF8_INPUT)
}

impl From<voicevox_core::AudioQueryOptions> for VoicevoxAudioQueryOptions {
    fn from(options: voicevox_core::AudioQueryOptions) -> Self {
        Self { kana: options.kana }
    }
}
impl From<VoicevoxAudioQueryOptions> for voicevox_core::AudioQueryOptions {
    fn from(options: VoicevoxAudioQueryOptions) -> Self {
        Self { kana: options.kana }
    }
}

impl From<VoicevoxSynthesisOptions> for voicevox_core::SynthesisOptions {
    fn from(options: VoicevoxSynthesisOptions) -> Self {
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

impl Default for VoicevoxInitializeOptions {
    fn default() -> Self {
        let options = voicevox_core::InitializeOptions::default();
        Self {
            use_gpu: options.use_gpu,
            cpu_num_threads: options.cpu_num_threads,
            load_all_models: options.load_all_models,
            open_jtalk_dict_dir: null(),
        }
    }
}

impl VoicevoxInitializeOptions {
    pub(crate) unsafe fn try_into_options(
        self,
    ) -> std::result::Result<voicevox_core::InitializeOptions, VoicevoxResultCode> {
        let open_jtalk_dict_dir = ensure_utf8(CStr::from_ptr(self.open_jtalk_dict_dir))?;
        Ok(voicevox_core::InitializeOptions {
            use_gpu: self.use_gpu,
            cpu_num_threads: self.cpu_num_threads,
            load_all_models: self.load_all_models,
            open_jtalk_dict_dir: Some(PathBuf::from(open_jtalk_dict_dir)),
        })
    }
}

impl From<voicevox_core::TtsOptions> for VoicevoxTtsOptions {
    fn from(options: voicevox_core::TtsOptions) -> Self {
        Self {
            kana: options.kana,
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

impl From<VoicevoxTtsOptions> for voicevox_core::TtsOptions {
    fn from(options: VoicevoxTtsOptions) -> Self {
        Self {
            kana: options.kana,
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}

impl Default for VoicevoxSynthesisOptions {
    fn default() -> Self {
        let options = voicevox_core::TtsOptions::default();
        Self {
            enable_interrogative_upspeak: options.enable_interrogative_upspeak,
        }
    }
}
