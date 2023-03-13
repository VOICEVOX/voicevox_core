mod c_impls;
/// cbindgen:ignore
mod compatible_engine;
mod helpers;
use self::helpers::*;
use c_impls::*;
use chrono::SecondsFormat;
use is_terminal::IsTerminal;
use libc::{c_void, free, malloc, memcpy};
use once_cell::sync::Lazy;
use std::env;
use std::ffi::{CStr, CString};
use std::fmt;
use std::io::{self, Write};
use std::mem::size_of;
use std::os::raw::c_char;
use std::sync::{Mutex, MutexGuard};
use tokio::runtime::Handle;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::EnvFilter;
use voicevox_core::StyleId;
use voicevox_core::{
    AccentPhraseModel, AudioQueryModel, AudioQueryOptions, TtsOptions, VoiceModelId,
};
use voicevox_core::{SupportedDevices, SynthesisOptions};

#[cfg(test)]
use rstest::*;

static _INTERNAL: Lazy<()> = Lazy::new(|| {
    let _ = init_logger();

    fn init_logger() -> std::result::Result<(), impl Sized> {
        tracing_subscriber::fmt()
            .with_env_filter(if env::var_os(EnvFilter::DEFAULT_ENV).is_some() {
                EnvFilter::from_default_env()
            } else {
                "error,voicevox_core=info,voicevox_core_c_api=info,onnxruntime=info".into()
            })
            .with_timer(local_time as fn(&mut Writer<'_>) -> _)
            .with_ansi(out().is_terminal() && env_allows_ansi())
            .with_writer(out)
            .try_init()
    }

    fn local_time(wtr: &mut Writer<'_>) -> fmt::Result {
        // ローカル時刻で表示はするが、そのフォーマットはtracing-subscriber本来のものに近いようにする。
        // https://github.com/tokio-rs/tracing/blob/tracing-subscriber-0.3.16/tracing-subscriber/src/fmt/time/datetime.rs#L235-L241
        wtr.write_str(&chrono::Local::now().to_rfc3339_opts(SecondsFormat::Micros, false))
    }

    fn out() -> impl IsTerminal + Write {
        io::stderr()
    }

    fn env_allows_ansi() -> bool {
        // https://docs.rs/termcolor/1.2.0/src/termcolor/lib.rs.html#245-291
        // ただしWindowsではPowerShellっぽかったらそのまま許可する。
        // ちゃんとやるなら`ENABLE_VIRTUAL_TERMINAL_PROCESSING`をチェックするなり、そもそも
        // fwdansiとかでWin32の色に変換するべきだが、面倒。
        env::var_os("TERM").map_or(
            cfg!(windows) && env::var_os("PSModulePath").is_some(),
            |term| term != "dumb",
        ) && env::var_os("NO_COLOR").is_none()
    }
});

/*
 * Cの関数として公開するための型や関数を定義するこれらの実装はvoicevox_core/publish.rsに定義してある対応する関数にある
 * この関数ではvoicevox_core/publish.rsにある対応する関数の呼び出しと、その戻り値をCの形式に変換する処理のみとする
 * これはC文脈の処理と実装をわけるためと、内部実装の変更がAPIに影響を与えにくくするためである
 * voicevox_core/publish.rsにある対応する関数とはこのファイルに定義してある公開関数からvoicevoxプレフィックスを取り除いた名前の関数である
 */

/// 参照カウントで管理されたOpenJtalk
pub struct OpenJtalkRc;

/// 参照カウントで管理されたOpenJtalkを生成する
///
/// # Safety
/// @out_open_jtalk 自動でheap領域が割り当てられるため :voicevox_open_jtalk_rc_delete で開放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_open_jtalk_rc_new(
    open_jtalk_dic_dir: *const c_char,
    out_open_jtalk: *mut *mut OpenJtalkRc,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let open_jtalk_dic_dir = ensure_utf8(CStr::from_ptr(open_jtalk_dic_dir))?;
        let mut open_jtalk = COpenJtalkRc::new_with_initialize(open_jtalk_dic_dir)?;
        let o = malloc(size_of::<COpenJtalkRc>());

        memcpy(
            o,
            &mut open_jtalk as *mut COpenJtalkRc as *mut c_void,
            size_of::<COpenJtalkRc>(),
        );
        out_open_jtalk.write(o as *mut OpenJtalkRc);
        Ok(())
    })())
}

/// 参照カウントで管理されたOpenJtalkを削除する
/// @param [in] open_jtalk 参照カウントで管理されたOpenJtalk
///
/// # Safety
/// @open_jtalk 有効な :OpenJtalkRc のポインタであること
#[no_mangle]
pub unsafe extern "C" fn voicevox_open_jtalk_rc_delete(open_jtalk: *mut OpenJtalkRc) {
    let _ = *(open_jtalk as *mut COpenJtalkRc);
    free(open_jtalk as *mut c_void)
}

pub use voicevox_core::result_code::VoicevoxResultCode;

/// ハードウェアアクセラレーションモードを設定する設定値
#[repr(i32)]
#[derive(Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum VoicevoxAccelerationMode {
    /// 実行環境に合った適切なハードウェアアクセラレーションモードを選択する
    VOICEVOX_ACCELERATION_MODE_AUTO = 0,
    /// ハードウェアアクセラレーションモードを"CPU"に設定する
    VOICEVOX_ACCELERATION_MODE_CPU = 1,
    /// ハードウェアアクセラレーションモードを"GPU"に設定する
    VOICEVOX_ACCELERATION_MODE_GPU = 2,
}

/// 初期化オプション
#[repr(C)]
pub struct VoicevoxInitializeOptions {
    /// ハードウェアアクセラレーションモード
    acceleration_mode: VoicevoxAccelerationMode,
    /// CPU利用数を指定
    /// 0を指定すると環境に合わせたCPUが利用される
    cpu_num_threads: u16,
    /// 全てのモデルを読み込む
    load_all_models: bool,
}

/// デフォルトの初期化オプションを生成する
/// @return デフォルト値が設定された初期化オプション
#[no_mangle]
pub extern "C" fn voicevox_make_default_initialize_options() -> VoicevoxInitializeOptions {
    VoicevoxInitializeOptions::default()
}

static VOICEVOX_VERSION: once_cell::sync::Lazy<CString> =
    once_cell::sync::Lazy::new(|| CString::new(voicevox_core::get_version()).unwrap());

/// voicevoxのバージョンを取得する
/// @return SemVerでフォーマットされたバージョン
#[no_mangle]
pub extern "C" fn voicevox_get_version() -> *const c_char {
    VOICEVOX_VERSION.as_ptr()
}

/// 音声合成モデル
#[repr(C)]
pub struct VoicevoxVoiceModel;

/// 音声合成モデルID
pub type VoicevoxVoiceModelId = *const c_char;

/// スタイルID
pub type VoicevoxStyleId = u32;

/// vvmファイルパスから音声モデルを生成する
/// @param [in] path vvmファイルパス
/// @param [out] out_model 新しく生成された音声モデルの出力先
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param path null終端文字列であること
/// @param out_model 自動でheapメモリが割り当てられるので ::voicevox_model_delete で解放する必要がある
pub unsafe extern "C" fn voicevox_model_new_from_path(
    path: *const c_char,
    out_model: *mut *mut VoicevoxVoiceModel,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        unsafe {
            let mut model = Handle::current()
                .block_on(CVoiceModel::from_path(ensure_utf8(CStr::from_ptr(path))?))?;
            let m = malloc(size_of::<CVoiceModel>());
            memcpy(
                m,
                &mut model as *mut CVoiceModel as *mut c_void,
                size_of::<CVoiceModel>(),
            );
            out_model.write(m as *mut VoicevoxVoiceModel);
        }
        Ok(())
    })())
}

/// 音声モデルのIDを取得する
/// @param [in] model 音声モデル #VoicevoxVoiceModel
/// @return 音声モデルID #VoicevoxVoiceModelId
///
/// # Safety
/// @param model 有効な #VoicevoxVoiceModel へのポインタであること
pub unsafe extern "C" fn voicevox_model_id(
    model: *const VoicevoxVoiceModel,
) -> VoicevoxVoiceModelId {
    let model = &*(model as *const CVoiceModel);
    model.id().as_ptr()
}

/// 音声モデルのメタ情報を取得する
/// @param [in] model 音声モデル #VoicevoxVoiceModel
/// @return メタ情報のjson文字列
///
/// # Safety
/// @param model 有効な #VoicevoxVoiceModel へのポインタであること
pub unsafe extern "C" fn voicevox_model_get_metas_json(
    model: *const VoicevoxVoiceModel,
) -> *const c_char {
    let model = &*(model as *const CVoiceModel);
    model.metas().as_ptr()
}

/// 音声モデルを破棄する
/// @param [in] model 破棄する音声モデル #VoicevoxVoiceModel
///
/// # Safety
/// @param model 有効な #VoicevoxVoiceModel へのポインタであること
pub unsafe extern "C" fn voicevox_model_delete(model: *mut VoicevoxVoiceModel) {
    let _ = *(model as *mut CVoiceModel);
    free(model as *mut c_void);
}

#[repr(C)]
pub struct VoicevoxVoiceSynthesizer;

/// 音声シンセサイザを生成して初期化する
/// @param [in] open_jtalk 参照カウントで管理されたOpenJtalk
/// @param [in] options 初期化オプション #VoicevoxInitializeOptions
/// @param [out] out_synthesizer 新しく生成された音声シンセサイザの出力先 #VoicevoxVoiceSynthesizer
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param out_synthesizer 自動でheapメモリが割り当てられるので ::voicevox_synthesizer_delete で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_new_with_initialize(
    open_jtalk: *const OpenJtalkRc,
    options: VoicevoxInitializeOptions,
    out_synthesizer: *mut *mut VoicevoxVoiceSynthesizer,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let options = options.into();
        let handle = Handle::current();
        let open_jtalk = &*(open_jtalk as *const COpenJtalkRc);

        let synthesizer =
            handle.block_on(CVoiceSynthesizer::new_with_initialize(open_jtalk, &options))?;
        unsafe {
            let s = malloc(size_of::<CVoiceSynthesizer>());
            memcpy(
                s,
                (&synthesizer as *const CVoiceSynthesizer) as *const c_void,
                size_of::<CVoiceSynthesizer>(),
            );
            *out_synthesizer = s as *mut VoicevoxVoiceSynthesizer;
        }
        Ok(())
    })())
}

/// 音声シンセサイザを破棄する
/// @param [in] synthesizer 破棄する音声シンセサイザ #VoicevoxVoiceSynthesizer
///
/// # Safety
/// @param synthesizer 有効な #VoicevoxVoiceSynthesizer へのポインタであること
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_delete(synthesizer: *mut VoicevoxVoiceSynthesizer) {
    let _ = *(synthesizer as *const CVoiceSynthesizer);
    free(synthesizer as *mut c_void);
}

/// モデルを読み込む
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] model 音声モデル
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param synthesizer 有効な #VoicevoxVoiceSynthesizer へのポインタであること
/// @param model 有効な #VoicevoxVoiceModel へのポインタであること
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_load_model(
    synthesizer: *mut VoicevoxVoiceSynthesizer,
    model: *const VoicevoxVoiceModel,
) -> VoicevoxResultCode {
    let synthesizer = &mut *(synthesizer as *mut CVoiceSynthesizer);
    let model = &*(model as *const CVoiceModel);
    into_result_code_with_error(
        Handle::current()
            .block_on(synthesizer.load_model(model.model()))
            .map_err(Into::into),
    )
}

/// モデルの読み込みを解除する
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] model_id 音声モデルID
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param synthesizer 有効な #VoicevoxVoiceSynthesizer へのポインタであること
/// @param model_id NULL終端文字列であること
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_unload_model(
    synthesizer: *mut VoicevoxVoiceSynthesizer,
    model_id: VoicevoxVoiceModelId,
) -> VoicevoxResultCode {
    let synthesizer = &mut *(synthesizer as *mut CVoiceSynthesizer);
    into_result_code_with_error((|| {
        let raw_model_id = ensure_utf8(unsafe { CStr::from_ptr(model_id) })?;
        synthesizer
            .unload_model(&VoiceModelId::new(raw_model_id.to_string()))
            .map_err(Into::into)
    })())
}

/// ハードウェアアクセラレーションがGPUモードか判定する
/// @param [in] synthesizer 音声シンセサイザ
/// @return GPUモードならtrue、そうでないならfalse
///
/// # Safety
/// @param synthesizer 有効な #VoicevoxVoiceSynthesizer へのポインタであること
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_is_gpu_mode(
    synthesizer: *const VoicevoxVoiceSynthesizer,
) -> bool {
    let synthesizer = &*(synthesizer as *const CVoiceSynthesizer);
    synthesizer.synthesizer().is_gpu_mode()
}

/// 指定したspeaker_idのモデルが読み込まれているか判定する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxVoiceSynthesizer
/// @param [in] model_id 音声モデルのID #VoicevoxVoiceModelId
/// @return モデルが読み込まれているのであればtrue、そうでないならfalse
///
/// # Safety
/// @param synthesizer 有効な #VoicevoxVoiceSynthesizer へのポインタであること
/// @param model_id NULL終端文字列
#[no_mangle]
pub unsafe extern "C" fn voicevox_is_loaded_model(
    synthesizer: *const VoicevoxVoiceSynthesizer,
    model_id: VoicevoxVoiceModelId,
) -> bool {
    let synthesizer = &*(synthesizer as *const CVoiceSynthesizer);
    let raw_model_id = ensure_utf8(unsafe { CStr::from_ptr(model_id) }).unwrap();
    synthesizer
        .synthesizer()
        .is_loaded_model(&VoiceModelId::new(raw_model_id.into()))
}

/// メタ情報をjsonで取得する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxVoiceSynthesizer
/// @return メタ情報のjson文字列
///
/// # Safety
/// @param synthesizer 有効な #VoicevoxVoiceSynthesizer へのポインタであること
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_get_metas_json(
    synthesizer: *const VoicevoxVoiceSynthesizer,
) -> *const c_char {
    let synthesizer = &*(synthesizer as *const CVoiceSynthesizer);
    synthesizer.metas().as_ptr()
}

static VOICEVOX_SUPPORTED_DEVICES_JSON: once_cell::sync::Lazy<CString> =
    once_cell::sync::Lazy::new(|| {
        CString::new(
            serde_json::to_string(&SupportedDevices::get_supported_devices().unwrap()).unwrap(),
        )
        .unwrap()
    });

/// サポートデバイス情報をjsonで取得する
/// @return サポートデバイス情報のjson文字列
#[no_mangle]
pub extern "C" fn voicevox_get_supported_devices_json() -> *const c_char {
    VOICEVOX_SUPPORTED_DEVICES_JSON.as_ptr()
}

/// Audio query のオプション
#[repr(C)]
pub struct VoicevoxAudioQueryOptions {
    /// aquestalk形式のkanaとしてテキストを解釈する
    kana: bool,
}

/// デフォルトの AudioQuery のオプションを生成する
/// @return デフォルト値が設定された AudioQuery オプション
#[no_mangle]
pub extern "C" fn voicevox_make_default_audio_query_options() -> VoicevoxAudioQueryOptions {
    voicevox_core::AudioQueryOptions::default().into()
}

/// AudioQuery を実行する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxVoiceSynthesizer
/// @param [in] text テキスト
/// @param [in] style_id スタイルID #VoicevoxStyleId
/// @param [in] options AudioQueryのオプション #VoicevoxAudioQueryOptions
/// @param [out] output_audio_query_json AudioQuery を json でフォーマットしたもの
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param text null終端文字列であること
/// @param output_audio_query_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_audio_query(
    synthesizer: *const VoicevoxVoiceSynthesizer,
    text: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxAudioQueryOptions,
    output_audio_query_json: *mut *mut c_char,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let synthesizer = &*(synthesizer as *const CVoiceSynthesizer);
        let text = CStr::from_ptr(text);
        let japanese_or_kana = ensure_utf8(text)?;
        let audio_query = Handle::current().block_on(synthesizer.synthesizer().audio_query(
            japanese_or_kana,
            &StyleId::new(style_id),
            &AudioQueryOptions::from(options),
        ))?;
        let audio_query = CString::new(audio_query_model_to_json(&audio_query))
            .expect("should not contain '\\0'");
        write_json_to_ptr(output_audio_query_json, &audio_query);
        Ok(())
    })())
}

/// create_accent_phrases を実行する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxVoiceSynthesizer
/// @param [in] text テキスト
/// @param [in] style_id スタイルID #VoicevoxStyleId
/// @param [in] output_accent_phrases_json アクセントフレーズのjson文字列
///
/// # Safety
/// @param text null終端文字列であること
/// @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_create_accent_phrases(
    synthesizer: *const VoicevoxVoiceSynthesizer,
    text: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: *mut *mut c_char,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let synthesizer = &*(synthesizer as *const CVoiceSynthesizer);
        let text = ensure_utf8(CStr::from_ptr(text))?;
        let accent_phrases = Handle::current().block_on(
            synthesizer
                .synthesizer()
                .create_accent_phrases(text, &StyleId::new(style_id)),
        )?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        write_json_to_ptr(output_accent_phrases_json, &accent_phrases);
        Ok(())
    })())
}

/// replace_mora_data を実行する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxVoiceSynthesizer
/// @param [in] accent_phrases_json 変換前のアクセントフレーズのjson文字列
/// @param [in] style_id スタイルID #VoicevoxStyleId
/// @param [in] output_accent_phrases_json 変換後のアクセントフレーズのjson文字列
///
/// # Safety
/// @param accent_phrases_json null終端文字列であること
/// @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_replace_mora_data(
    synthesizer: *const VoicevoxVoiceSynthesizer,
    accent_phrases_json: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: *mut *mut c_char,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let synthesizer = &*(synthesizer as *const CVoiceSynthesizer);
        let accent_phrases: Vec<AccentPhraseModel> =
            serde_json::from_str(ensure_utf8(CStr::from_ptr(accent_phrases_json))?)
                .expect("invalid json\0");
        let accent_phrases = Handle::current().block_on(
            synthesizer
                .synthesizer()
                .replace_mora_data(&accent_phrases, &StyleId::new(style_id)),
        )?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        write_json_to_ptr(output_accent_phrases_json, &accent_phrases);
        Ok(())
    })())
}

/// replace_phoneme_length を実行する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxVoiceSynthesizer
/// @param [in] accent_phrases_json 変換前のアクセントフレーズのjson文字列
/// @param [in] style_id スタイルID #VoicevoxStyleId
/// @param [in] output_accent_phrases_json 変換後のアクセントフレーズのjson文字列
///
/// # Safety
/// @param accent_phrases_json null終端文字列であること
/// @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_replace_phoneme_length(
    synthesizer: *const VoicevoxVoiceSynthesizer,
    accent_phrases_json: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: *mut *mut c_char,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let synthesizer = &*(synthesizer as *const CVoiceSynthesizer);
        let accent_phrases: Vec<AccentPhraseModel> =
            serde_json::from_str(ensure_utf8(CStr::from_ptr(accent_phrases_json))?)
                .expect("invalid json\0");
        let accent_phrases = Handle::current().block_on(
            synthesizer
                .synthesizer()
                .replace_phoneme_length(&accent_phrases, &StyleId::new(style_id)),
        )?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        write_json_to_ptr(output_accent_phrases_json, &accent_phrases);
        Ok(())
    })())
}

/// `voicevox_synthesizer_synthesis` のオプション
#[repr(C)]
pub struct VoicevoxSynthesisOptions {
    /// 疑問文の調整を有効にする
    enable_interrogative_upspeak: bool,
}

/// デフォルトの `voicevox_synthesizer_synthesis` のオプションを生成する
/// @return デフォルト値が設定された `voicevox_synthesizer_synthesis` のオプション
#[no_mangle]
pub extern "C" fn voicevox_make_default_synthesis_options() -> VoicevoxSynthesisOptions {
    VoicevoxSynthesisOptions::default()
}

/// AudioQuery から音声合成する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxVoiceSynthesizer
/// @param [in] audio_query_json jsonフォーマットされた AudioQuery
/// @param [in] style_id スタイルID #VoicevoxStyleId
/// @param [in] options AudioQueryから音声合成オプション
/// @param [out] output_wav_length 出力する wav データのサイズ
/// @param [out] output_wav wav データの出力先
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param output_wav_length 出力先の領域が確保された状態でpointerに渡されていること
/// @param output_wav 自動で output_wav_length 分のデータが割り当てられるので ::voicevox_wav_free で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_synthesis(
    synthesizer: *const VoicevoxVoiceSynthesizer,
    audio_query_json: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxSynthesisOptions,
    output_wav_length: *mut usize,
    output_wav: *mut *mut u8,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let synthesizer = &*(synthesizer as *const CVoiceSynthesizer);
        let audio_query_json = CStr::from_ptr(audio_query_json)
            .to_str()
            .map_err(|_| CApiError::InvalidUtf8Input)?;
        let audio_query: AudioQueryModel =
            serde_json::from_str(audio_query_json).map_err(CApiError::InvalidAudioQuery)?;
        let wav = &Handle::current().block_on(synthesizer.synthesizer().synthesis(
            &audio_query,
            &StyleId::new(style_id),
            &SynthesisOptions::from(options),
        ))?;
        write_wav_to_ptr(output_wav, output_wav_length, wav);
        Ok(())
    })())
}

/// テキスト音声合成オプション
#[repr(C)]
pub struct VoicevoxTtsOptions {
    /// aquestalk形式のkanaとしてテキストを解釈する
    kana: bool,
    /// 疑問文の調整を有効にする
    enable_interrogative_upspeak: bool,
}

/// デフォルトのテキスト音声合成オプションを生成する
/// @return テキスト音声合成オプション
#[no_mangle]
pub extern "C" fn voicevox_make_default_tts_options() -> VoicevoxTtsOptions {
    voicevox_core::TtsOptions::default().into()
}

/// テキスト音声合成を実行する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxVoiceSynthesizer
/// @param [in] text テキスト
/// @param [in] style_id スタイルID #VoicevoxStyleId
/// @param [in] options テキスト音声合成オプション
/// @param [out] output_wav_length 出力する wav データのサイズ
/// @param [out] output_wav wav データの出力先
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param output_wav_length 出力先の領域が確保された状態でpointerに渡されていること
/// @param output_wav は自動で output_wav_length 分のデータが割り当てられるので ::voicevox_wav_free で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_tts(
    synthesizer: *const VoicevoxVoiceSynthesizer,
    text: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxTtsOptions,
    output_wav_length: *mut usize,
    output_wav: *mut *mut u8,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let synthesizer = &*(synthesizer as *const CVoiceSynthesizer);
        let text = ensure_utf8(CStr::from_ptr(text))?;
        let output = Handle::current().block_on(synthesizer.synthesizer().tts(
            text,
            &StyleId::new(style_id),
            &TtsOptions::from(options),
        ))?;
        write_wav_to_ptr(output_wav, output_wav_length, output.as_slice());
        Ok(())
    })())
}

/// jsonフォーマットされた AudioQuery データのメモリを解放する
/// @param [in] json 解放する json データ
///
/// # Safety
/// @param json 確保したメモリ領域が破棄される
#[no_mangle]
pub unsafe extern "C" fn voicevox_json_free(json: *mut c_char) {
    libc::free(json as *mut c_void);
}

/// wav データのメモリを解放する
/// @param [in] wav 解放する wav データ
///
/// # Safety
/// @param wav 確保したメモリ領域が破棄される
#[no_mangle]
pub unsafe extern "C" fn voicevox_wav_free(wav: *mut u8) {
    libc::free(wav as *mut c_void);
}

/// エラー結果をメッセージに変換する
/// @param [in] result_code メッセージに変換する result_code
/// @return 結果コードを元に変換されたメッセージ文字列
#[no_mangle]
pub extern "C" fn voicevox_error_result_to_message(
    result_code: VoicevoxResultCode,
) -> *const c_char {
    voicevox_core::result_code::error_result_to_message(result_code).as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use pretty_assertions::assert_eq;
    use voicevox_core::Error;
    use voicevox_core::Result;

    #[rstest]
    #[case(Ok(()), VoicevoxResultCode::VOICEVOX_RESULT_OK)]
    #[case(
        Err(Error::NotLoadedOpenjtalkDict),
        VoicevoxResultCode::VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR
    )]
    #[case(
        Err(Error::LoadModel {
            path: "path/to/model.onnx".into(),
            source: anyhow!("some load model error"),
        }),
        VoicevoxResultCode::VOICEVOX_RESULT_LOAD_MODEL_ERROR
    )]
    #[case(
        Err(Error::GetSupportedDevices(anyhow!("some get supported devices error"))),
        VoicevoxResultCode::VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR
    )]
    fn into_result_code_with_error_works(
        #[case] result: Result<()>,
        #[case] expected: VoicevoxResultCode,
    ) {
        let actual = into_result_code_with_error(result.map_err(Into::into));
        assert_eq!(expected, actual);
    }
}
