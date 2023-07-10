mod c_impls;
/// cbindgen:ignore
mod compatible_engine;
mod drop_check;
mod helpers;
mod slice_owner;
use self::drop_check::C_STRING_DROP_CHECKER;
use self::helpers::*;
use self::slice_owner::U8_SLICE_OWNER;
use chrono::SecondsFormat;
use const_default::ConstDefault;
use derive_getters::Getters;
use once_cell::sync::Lazy;
use std::env;
use std::ffi::{CStr, CString};
use std::fmt;
use std::io::{self, IsTerminal, Write};
use std::os::raw::c_char;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::runtime::Runtime;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::EnvFilter;
use voicevox_core::{
    AccentPhraseModel, AudioQueryModel, AudioQueryOptions, OpenJtalk, TtsOptions, UserDictWord,
    VoiceModel, VoiceModelId,
};
use voicevox_core::{StyleId, SupportedDevices, SynthesisOptions, Synthesizer};

#[cfg(test)]
use rstest::*;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
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
    Runtime::new().unwrap()
});

/*
 * Cの関数として公開するための型や関数を定義するこれらの実装はvoicevox_core/publish.rsに定義してある対応する関数にある
 * この関数ではvoicevox_core/publish.rsにある対応する関数の呼び出しと、その戻り値をCの形式に変換する処理のみとする
 * これはC文脈の処理と実装をわけるためと、内部実装の変更がAPIに影響を与えにくくするためである
 * voicevox_core/publish.rsにある対応する関数とはこのファイルに定義してある公開関数からvoicevoxプレフィックスを取り除いた名前の関数である
 */

/// 参照カウントで管理されたOpenJtalk
pub struct OpenJtalkRc {
    open_jtalk: Arc<OpenJtalk>,
}

/// 参照カウントで管理されたOpenJtalkを生成する
///
/// # Safety
/// @out_open_jtalk 自動でheap領域が割り当てられるため :voicevox_open_jtalk_rc_delete で開放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_open_jtalk_rc_new(
    open_jtalk_dic_dir: *const c_char,
    out_open_jtalk: NonNull<Box<OpenJtalkRc>>,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let open_jtalk_dic_dir = ensure_utf8(CStr::from_ptr(open_jtalk_dic_dir))?;
        let open_jtalk = OpenJtalkRc::new_with_initialize(open_jtalk_dic_dir)?.into();
        out_open_jtalk.as_ptr().write_unaligned(open_jtalk);
        Ok(())
    })())
}

/// OpenJtalkにユーザー辞書を読み込ませる。
/// この関数を呼び出した後にユーザー辞書を変更した場合、再度この関数を呼び出す必要がある。
/// @param [in] open_jtalk 参照カウントで管理されたOpenJtalk
/// @param [in] user_dict ユーザー辞書
///
/// # Safety
/// @open_jtalk 有効な :OpenJtalkRc のポインタであること
/// @user_dict 有効な :VoicevoxUserDict のポインタであること
#[no_mangle]
pub extern "C" fn voicevox_open_jtalk_rc_load_user_dict(
    open_jtalk: &mut OpenJtalkRc,
    user_dict: &VoicevoxUserDict,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let user_dict = user_dict.to_owned();
        {
            let dict = user_dict.dict.as_ref().lock().expect("lock failed");
            open_jtalk.open_jtalk.load_user_dict(&dict)?;
        }
        Ok(())
    })())
}

/// 参照カウントで管理されたOpenJtalkを削除する
/// @param [in] open_jtalk 参照カウントで管理されたOpenJtalk
///
/// # Safety
/// @open_jtalk 有効な :OpenJtalkRc のポインタであること
#[no_mangle]
pub extern "C" fn voicevox_open_jtalk_rc_delete(open_jtalk: Box<OpenJtalkRc>) {
    drop(open_jtalk);
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

/// デフォルトの初期化オプション
#[no_mangle]
pub static voicevox_default_initialize_options: VoicevoxInitializeOptions = ConstDefault::DEFAULT;

/// voicevoxのバージョン
#[no_mangle]
pub static voicevox_version: &c_char = {
    const VOICEVOX_VERSION: &CStr = unsafe {
        // SAFETY: The package version is a SemVer, so it should not contain '\0'
        CStr::from_bytes_with_nul_unchecked(concat!(env!("CARGO_PKG_VERSION"), '\0').as_bytes())
    };

    // SAFETY: `CStr::as_ptr` always returns a valid pointer.
    unsafe { &*VOICEVOX_VERSION.as_ptr() }
};

/// 音声モデル
#[derive(Getters)]
pub struct VoicevoxVoiceModel {
    model: VoiceModel,
    id: CString,
    metas: CString,
}

/// 音声モデルID
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
/// @param out_model 自動でheapメモリが割り当てられるので ::voicevox_voice_model_delete で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_voice_model_new_from_path(
    path: *const c_char,
    out_model: NonNull<Box<VoicevoxVoiceModel>>,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let path = ensure_utf8(CStr::from_ptr(path))?;
        let model = RUNTIME
            .block_on(VoicevoxVoiceModel::from_path(path))?
            .into();
        out_model.as_ptr().write_unaligned(model);
        Ok(())
    })())
}

/// 音声モデルのIDを取得する
/// @param [in] model 音声モデル #VoicevoxVoiceModel
/// @return 音声モデルID #VoicevoxVoiceModelId
///
/// # Safety
/// @param model 有効な #VoicevoxVoiceModel へのポインタであること
#[no_mangle]
pub extern "C" fn voicevox_voice_model_id(model: &VoicevoxVoiceModel) -> VoicevoxVoiceModelId {
    model.id().as_ptr()
}

/// 音声モデルのメタ情報を取得する
/// @param [in] model 音声モデル #VoicevoxVoiceModel
/// @return メタ情報のjson文字列
///
/// # Safety
/// @param model 有効な #VoicevoxVoiceModel へのポインタであること
#[no_mangle]
pub extern "C" fn voicevox_voice_model_get_metas_json(model: &VoicevoxVoiceModel) -> *const c_char {
    model.metas().as_ptr()
}

/// 音声モデルを破棄する
/// @param [in] model 破棄する音声モデル #VoicevoxVoiceModel
///
/// # Safety
/// @param model 有効な #VoicevoxVoiceModel へのポインタであること
#[no_mangle]
pub extern "C" fn voicevox_voice_model_delete(model: Box<VoicevoxVoiceModel>) {
    drop(model);
}

#[derive(Getters)]
pub struct VoicevoxSynthesizer {
    synthesizer: Synthesizer,
    metas_cstring: CString,
}

/// 音声シンセサイザを生成して初期化する
/// @param [in] open_jtalk 参照カウントで管理されたOpenJtalk
/// @param [in] options 初期化オプション #VoicevoxInitializeOptions
/// @param [out] out_synthesizer 新しく生成された音声シンセサイザの出力先 #VoicevoxSynthesizer
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param out_synthesizer 自動でheapメモリが割り当てられるので ::voicevox_synthesizer_delete で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_new_with_initialize(
    open_jtalk: &OpenJtalkRc,
    options: VoicevoxInitializeOptions,
    out_synthesizer: NonNull<Box<VoicevoxSynthesizer>>,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let options = options.into();

        let synthesizer = RUNTIME
            .block_on(VoicevoxSynthesizer::new_with_initialize(
                open_jtalk, &options,
            ))?
            .into();
        out_synthesizer.as_ptr().write_unaligned(synthesizer);
        Ok(())
    })())
}

/// 音声シンセサイザを破棄する
/// @param [in] synthesizer 破棄する音声シンセサイザ #VoicevoxSynthesizer
///
/// # Safety
/// @param synthesizer 有効な #VoicevoxSynthesizer へのポインタであること
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_delete(synthesizer: Box<VoicevoxSynthesizer>) {
    drop(synthesizer);
}

/// モデルを読み込む
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] model 音声モデル
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param synthesizer 有効な #VoicevoxSynthesizer へのポインタであること
/// @param model 有効な #VoicevoxVoiceModel へのポインタであること
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_load_voice_model(
    synthesizer: &mut VoicevoxSynthesizer,
    model: &VoicevoxVoiceModel,
) -> VoicevoxResultCode {
    into_result_code_with_error(
        RUNTIME
            .block_on(synthesizer.load_voice_model(model.model()))
            .map_err(Into::into),
    )
}

/// モデルの読み込みを解除する
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] model_id 音声モデルID
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param synthesizer 有効な #VoicevoxSynthesizer へのポインタであること
/// @param model_id NULL終端文字列であること
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_unload_voice_model(
    synthesizer: &mut VoicevoxSynthesizer,
    model_id: VoicevoxVoiceModelId,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let raw_model_id = ensure_utf8(unsafe { CStr::from_ptr(model_id) })?;
        synthesizer
            .unload_voice_model(&VoiceModelId::new(raw_model_id.to_string()))
            .map_err(Into::into)
    })())
}

/// ハードウェアアクセラレーションがGPUモードか判定する
/// @param [in] synthesizer 音声シンセサイザ
/// @return GPUモードならtrue、そうでないならfalse
///
/// # Safety
/// @param synthesizer 有効な #VoicevoxSynthesizer へのポインタであること
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_is_gpu_mode(synthesizer: &VoicevoxSynthesizer) -> bool {
    synthesizer.synthesizer().is_gpu_mode()
}

/// 指定したspeaker_idのモデルが読み込まれているか判定する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
/// @param [in] model_id 音声モデルのID #VoicevoxVoiceModelId
/// @return モデルが読み込まれているのであればtrue、そうでないならfalse
///
/// # Safety
/// @param synthesizer 有効な #VoicevoxSynthesizer へのポインタであること
/// @param model_id NULL終端文字列
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_is_loaded_voice_model(
    synthesizer: &VoicevoxSynthesizer,
    model_id: VoicevoxVoiceModelId,
) -> bool {
    let raw_model_id = ensure_utf8(unsafe { CStr::from_ptr(model_id) }).unwrap();
    synthesizer
        .synthesizer()
        .is_loaded_voice_model(&VoiceModelId::new(raw_model_id.into()))
}

/// メタ情報をjsonで取得する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
/// @return メタ情報のjson文字列
///
/// # Safety
/// @param synthesizer 有効な #VoicevoxSynthesizer へのポインタであること
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_get_metas_json(
    synthesizer: &VoicevoxSynthesizer,
) -> *const c_char {
    synthesizer.metas().as_ptr()
}

/// サポートデバイス情報をjsonで取得する
/// @param [out] output_supported_devices_json サポートデバイス情報のjson文字列
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param output_supported_devices_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_create_supported_devices_json(
    output_supported_devices_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let supported_devices =
            CString::new(SupportedDevices::create()?.to_json().to_string()).unwrap();
        output_supported_devices_json.as_ptr().write_unaligned(
            C_STRING_DROP_CHECKER
                .whitelist(supported_devices)
                .into_raw(),
        );
        Ok(())
    })())
}

/// Audio query のオプション
#[repr(C)]
pub struct VoicevoxAudioQueryOptions {
    /// aquestalk形式のkanaとしてテキストを解釈する
    kana: bool,
}

/// デフォルトの AudioQuery のオプション
#[no_mangle]
pub static voicevox_default_audio_query_options: VoicevoxAudioQueryOptions = ConstDefault::DEFAULT;

/// AudioQuery を実行する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
/// @param [in] text テキスト。文字コードはUTF-8
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
    synthesizer: &VoicevoxSynthesizer,
    text: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxAudioQueryOptions,
    output_audio_query_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let text = CStr::from_ptr(text);
        let japanese_or_kana = ensure_utf8(text)?;
        let audio_query = RUNTIME.block_on(synthesizer.synthesizer().audio_query(
            japanese_or_kana,
            StyleId::new(style_id),
            &AudioQueryOptions::from(options),
        ))?;
        let audio_query = CString::new(audio_query_model_to_json(&audio_query))
            .expect("should not contain '\\0'");
        output_audio_query_json
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(audio_query).into_raw());
        Ok(())
    })())
}

/// `accent_phrases` のオプション
#[repr(C)]
pub struct VoicevoxAccentPhrasesOptions {
    /// aquestalk形式のkanaとしてテキストを解釈する
    kana: bool,
}

/// デフォルトの `accent_phrases` のオプション
#[no_mangle]
pub static voicevox_default_accent_phrases_options: VoicevoxAccentPhrasesOptions =
    ConstDefault::DEFAULT;

/// create_accent_phrases を実行する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
/// @param [in] text テキスト
/// @param [in] style_id スタイルID #VoicevoxStyleId
/// @param [in] output_accent_phrases_json アクセントフレーズのjson文字列
///
/// # Safety
/// @param text null終端文字列であること
/// @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_create_accent_phrases(
    synthesizer: &VoicevoxSynthesizer,
    text: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxAccentPhrasesOptions,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let text = ensure_utf8(CStr::from_ptr(text))?;
        let accent_phrases = RUNTIME.block_on(synthesizer.synthesizer().create_accent_phrases(
            text,
            StyleId::new(style_id),
            &options.into(),
        ))?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

/// replace_mora_data を実行する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
/// @param [in] accent_phrases_json 変換前のアクセントフレーズのjson文字列
/// @param [in] style_id スタイルID #VoicevoxStyleId
/// @param [in] output_accent_phrases_json 変換後のアクセントフレーズのjson文字列
///
/// # Safety
/// @param accent_phrases_json null終端文字列であること
/// @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_replace_mora_data(
    synthesizer: &VoicevoxSynthesizer,
    accent_phrases_json: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let accent_phrases: Vec<AccentPhraseModel> =
            serde_json::from_str(ensure_utf8(CStr::from_ptr(accent_phrases_json))?)
                .map_err(CApiError::InvalidAccentPhrase)?;
        let accent_phrases = RUNTIME.block_on(
            synthesizer
                .synthesizer()
                .replace_mora_data(&accent_phrases, StyleId::new(style_id)),
        )?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

/// replace_phoneme_length を実行する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
/// @param [in] accent_phrases_json 変換前のアクセントフレーズのjson文字列
/// @param [in] style_id スタイルID #VoicevoxStyleId
/// @param [in] output_accent_phrases_json 変換後のアクセントフレーズのjson文字列
///
/// # Safety
/// @param accent_phrases_json null終端文字列であること
/// @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_replace_phoneme_length(
    synthesizer: &VoicevoxSynthesizer,
    accent_phrases_json: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let accent_phrases: Vec<AccentPhraseModel> =
            serde_json::from_str(ensure_utf8(CStr::from_ptr(accent_phrases_json))?)
                .map_err(CApiError::InvalidAccentPhrase)?;
        let accent_phrases = RUNTIME.block_on(
            synthesizer
                .synthesizer()
                .replace_phoneme_length(&accent_phrases, StyleId::new(style_id)),
        )?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

/// replace_mora_pitch を実行する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
/// @param [in] accent_phrases_json 変換前のアクセントフレーズのjson文字列
/// @param [in] style_id スタイルID #VoicevoxStyleId
/// @param [in] output_accent_phrases_json 変換後のアクセントフレーズのjson文字列
///
/// # Safety
/// @param accent_phrases_json null終端文字列であること
/// @param output_accent_phrases_json 自動でheapメモリが割り当てられるので ::voicevox_json_free で解放する必要がある
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_replace_mora_pitch(
    synthesizer: &VoicevoxSynthesizer,
    accent_phrases_json: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let accent_phrases: Vec<AccentPhraseModel> =
            serde_json::from_str(ensure_utf8(CStr::from_ptr(accent_phrases_json))?)
                .map_err(CApiError::InvalidAccentPhrase)?;
        let accent_phrases = RUNTIME.block_on(
            synthesizer
                .synthesizer()
                .replace_mora_pitch(&accent_phrases, StyleId::new(style_id)),
        )?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

/// `voicevox_synthesizer_synthesis` のオプション
#[repr(C)]
pub struct VoicevoxSynthesisOptions {
    /// 疑問文の調整を有効にする
    enable_interrogative_upspeak: bool,
}

/// デフォルトの `voicevox_synthesizer_synthesis` のオプション
#[no_mangle]
pub static voicevox_default_synthesis_options: VoicevoxSynthesisOptions = ConstDefault::DEFAULT;

/// AudioQuery から音声合成する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
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
    synthesizer: &VoicevoxSynthesizer,
    audio_query_json: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxSynthesisOptions,
    output_wav_length: NonNull<usize>,
    output_wav: NonNull<*mut u8>,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let audio_query_json = CStr::from_ptr(audio_query_json)
            .to_str()
            .map_err(|_| CApiError::InvalidUtf8Input)?;
        let audio_query: AudioQueryModel =
            serde_json::from_str(audio_query_json).map_err(CApiError::InvalidAudioQuery)?;
        let wav = RUNTIME.block_on(synthesizer.synthesizer().synthesis(
            &audio_query,
            StyleId::new(style_id),
            &SynthesisOptions::from(options),
        ))?;
        U8_SLICE_OWNER.own_and_lend(wav, output_wav, output_wav_length);
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

/// デフォルトのテキスト音声合成オプション
#[no_mangle]
pub static voicevox_default_tts_options: VoicevoxTtsOptions = ConstDefault::DEFAULT;

/// テキスト音声合成を実行する
/// @param [in] synthesizer 音声シンセサイザ #VoicevoxSynthesizer
/// @param [in] text テキスト。文字コードはUTF-8
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
    synthesizer: &VoicevoxSynthesizer,
    text: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxTtsOptions,
    output_wav_length: NonNull<usize>,
    output_wav: NonNull<*mut u8>,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let text = ensure_utf8(CStr::from_ptr(text))?;
        let output = RUNTIME.block_on(synthesizer.synthesizer().tts(
            text,
            StyleId::new(style_id),
            &TtsOptions::from(options),
        ))?;
        U8_SLICE_OWNER.own_and_lend(output, output_wav, output_wav_length);
        Ok(())
    })())
}

/// jsonフォーマットされたデータのメモリを解放する
/// @param [in] json 解放する json データ
///
/// # Safety
/// @param voicevox_audio_query で確保されたポインタであり、かつ呼び出し側でバッファの変更を行われていないこと
#[no_mangle]
pub unsafe extern "C" fn voicevox_json_free(json: *mut c_char) {
    drop(CString::from_raw(C_STRING_DROP_CHECKER.check(json)));
}

/// wav データのメモリを解放する
/// @param [in] wav 解放する wav データ
///
/// # Safety
/// @param wav voicevox_tts,voicevox_synthesis で確保されたポインタであり、かつ呼び出し側でバッファの変更を行われていないこと
#[no_mangle]
pub extern "C" fn voicevox_wav_free(wav: *mut u8) {
    U8_SLICE_OWNER.drop_for(wav);
}

/// エラー結果をメッセージに変換する
/// @param [in] result_code メッセージに変換する result_code
/// @return 結果コードを元に変換されたメッセージ文字列
#[no_mangle]
pub extern "C" fn voicevox_error_result_to_message(
    result_code: VoicevoxResultCode,
) -> *const c_char {
    let message = CStr::from_bytes_with_nul(
        voicevox_core::result_code::error_result_to_message(result_code).as_ref(),
    )
    .expect("`error_result_to_message`が返す文字列はヌル終端であるはずである");

    C_STRING_DROP_CHECKER.blacklist(message).as_ptr()
}

/// ユーザー辞書
pub struct VoicevoxUserDict {
    dict: Arc<Mutex<voicevox_core::UserDict>>,
}

/// ユーザー辞書の単語
#[repr(C)]
pub struct VoicevoxUserDictWord {
    /// 表記
    surface: *const c_char,
    /// 読み
    pronunciation: *const c_char,
    /// アクセント型
    accent_type: usize,
    /// 単語の種類
    word_type: VoicevoxUserDictWordType,
    /// 優先度
    priority: u32,
}

/// ユーザー辞書の単語の種類
#[repr(i32)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum VoicevoxUserDictWordType {
    /// 固有名詞。
    VOICEVOX_USER_DICT_WORD_TYPE_PROPER_NOUN = 0,
    /// 一般名詞。
    VOICEVOX_USER_DICT_WORD_TYPE_COMMON_NOUN = 1,
    /// 動詞。
    VOICEVOX_USER_DICT_WORD_TYPE_VERB = 2,
    /// 形容詞。
    VOICEVOX_USER_DICT_WORD_TYPE_ADJECTIVE = 3,
    /// 接尾辞。
    VOICEVOX_USER_DICT_WORD_TYPE_SUFFIX = 4,
}

/// ユーザー辞書の単語のデフォルト値
#[no_mangle]
pub extern "C" fn voicevox_default_user_dict_word() -> VoicevoxUserDictWord {
    VoicevoxUserDictWord {
        surface: std::ptr::null(),
        pronunciation: std::ptr::null(),
        accent_type: UserDictWord::default().accent_type,
        word_type: UserDictWord::default().word_type.into(),
        priority: UserDictWord::default().priority,
    }
}

/// ユーザー辞書を作成する
/// @param [out] out_user_dict VoicevoxUserDictのポインタ
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param out_user_dict VoicevoxUserDictのポインタが有効な領域を指していること
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_new(
    out_user_dict: NonNull<Box<VoicevoxUserDict>>,
) -> VoicevoxResultCode {
    let dict = voicevox_core::UserDict::new();
    let user_dict = Box::new(VoicevoxUserDict {
        dict: Arc::new(Mutex::new(dict)),
    });
    out_user_dict.as_ptr().write(user_dict);

    VoicevoxResultCode::VOICEVOX_RESULT_OK
}

/// ユーザー辞書にファイルを読み込ませる
/// @param [in] user_dict VoicevoxUserDictのポインタ
/// @param [in] dict_path 読み込む辞書ファイルのパス
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param user_dict は有効な :VoicevoxUserDict のポインタであること
/// @param dict_path パスが有効な文字列を指していること
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_load(
    user_dict: &VoicevoxUserDict,
    dict_path: *const c_char,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let dict_path = ensure_utf8(unsafe { CStr::from_ptr(dict_path) })?;
        let mut dict = user_dict.dict.lock().unwrap();
        dict.load(dict_path)?;

        Ok(())
    })())
}

/// ユーザー辞書に単語を追加する
/// @param [in] user_dict VoicevoxUserDictのポインタ
/// @param [in] word 追加する単語
/// @param [out] out_word_uuid 追加した単語のUUID
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param user_dict は有効な :VoicevoxUserDict のポインタであること
/// @param word_uuid は呼び出し側で解放する必要がある
///
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_add_word(
    user_dict: &VoicevoxUserDict,
    word: &VoicevoxUserDictWord,
    out_word_uuid: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let word = word.try_into_word()?;
        let uuid = {
            let mut dict = user_dict.dict.lock().expect("lock failed");
            dict.add_word(word)?
        };
        let uuid = CString::new(uuid).expect("\\0を含まない文字列であることが保証されている");
        out_word_uuid
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(uuid).into_raw());

        Ok(())
    })())
}

/// ユーザー辞書の単語を更新する
/// @param [in] user_dict VoicevoxUserDictのポインタ
/// @param [in] word_uuid 更新する単語のUUID
/// @param [in] word 新しい単語のデータ
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param user_dict は有効な :VoicevoxUserDict のポインタであること
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_update_word(
    user_dict: &VoicevoxUserDict,
    word_uuid: *const u8,
    word: &VoicevoxUserDictWord,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let word_uuid = ensure_utf8(unsafe { CStr::from_ptr(word_uuid as *const c_char) })?;
        let word = word.try_into_word()?;
        {
            let mut dict = user_dict.dict.lock().expect("lock failed");
            dict.update_word(word_uuid, word)?;
        };

        Ok(())
    })())
}

/// ユーザー辞書から単語を削除する
/// @param [in] user_dict VoicevoxUserDictのポインタ
/// @param [in] word_uuid 削除する単語のUUID
/// @return 結果コード #VoicevoxResultCode
#[no_mangle]
pub extern "C" fn voicevox_user_dict_remove_word(
    user_dict: &VoicevoxUserDict,
    word_uuid: *const u8,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let word_uuid = ensure_utf8(unsafe { CStr::from_ptr(word_uuid as *const c_char) })?;
        {
            let mut dict = user_dict.dict.lock().expect("lock failed");
            dict.remove_word(word_uuid)?;
        };

        Ok(())
    })())
}

/// ユーザー辞書の単語をJSON形式で出力する
/// @param [in] user_dict VoicevoxUserDictのポインタ
/// @param [out] out_json JSON形式の文字列
/// @return 結果コード #VoicevoxResultCode
///
/// # Safety
/// @param user_dict は有効な :VoicevoxUserDict のポインタであること
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_get_json(
    user_dict: &VoicevoxUserDict,
    out_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    let dict = user_dict.dict.lock().expect("lock failed");
    let json = serde_json::to_string(&dict.words()).expect("should be always valid");
    let json = CString::new(json).expect("\\0を含まない文字列であることが保証されている");
    out_json
        .as_ptr()
        .write_unaligned(C_STRING_DROP_CHECKER.whitelist(json).into_raw());
    VoicevoxResultCode::VOICEVOX_RESULT_OK
}

/// 他のユーザー辞書をインポートする
/// @param [in] user_dict VoicevoxUserDictのポインタ
/// @param [in] other_dict インポートするユーザー辞書
/// @return 結果コード #VoicevoxResultCode
#[no_mangle]
pub extern "C" fn voicevox_user_dict_import(
    user_dict: &VoicevoxUserDict,
    other_dict: &VoicevoxUserDict,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        {
            let mut dict = user_dict.dict.lock().expect("lock failed");
            let other_dict = other_dict.dict.lock().expect("lock failed");
            dict.import(&other_dict)?;
        };

        Ok(())
    })())
}

/// ユーザー辞書をファイルに保存する
/// @param [in] user_dict VoicevoxUserDictのポインタ
/// @param [in] path 保存先のファイルパス
///
/// # Safety
/// @param user_dict は有効な :VoicevoxUserDict のポインタであること
/// @param path は有効なUTF-8文字列であること
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_save(
    user_dict: &VoicevoxUserDict,
    path: *const c_char,
) -> VoicevoxResultCode {
    into_result_code_with_error((|| {
        let path = ensure_utf8(CStr::from_ptr(path))?;
        {
            let dict = user_dict.dict.lock().expect("lock failed");
            dict.save(path)?;
        };

        Ok(())
    })())
}

/// ユーザー辞書を廃棄する。
/// @param [in] user_dict VoicevoxUserDictのポインタ
///
/// # Safety
/// @param user_dict は有効な :VoicevoxUserDict のポインタであること
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_delete(user_dict: Box<VoicevoxUserDict>) {
    drop(user_dict);
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
