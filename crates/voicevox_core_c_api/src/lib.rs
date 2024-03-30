// ここにあるRustdocはcbindgen向けのものである。safety documentation自体は書くが、Doxygenの慣習に従
// い`<dt>`で書く。
#![allow(clippy::missing_safety_doc)]

mod c_impls;
/// cbindgen:ignore
mod compatible_engine;
mod drop_check;
mod helpers;
mod result_code;
mod slice_owner;
use self::drop_check::C_STRING_DROP_CHECKER;
use self::helpers::{
    accent_phrases_to_json, audio_query_model_to_json, ensure_utf8, into_result_code_with_error,
    CApiError,
};
use self::result_code::VoicevoxResultCode;
use self::slice_owner::U8_SLICE_OWNER;
use anstream::{AutoStream, RawStream};
use chrono::SecondsFormat;
use colorchoice::ColorChoice;
use derive_getters::Getters;
use std::env;
use std::ffi::{CStr, CString};
use std::fmt;
use std::io;
use std::os::raw::c_char;
use std::ptr::NonNull;
use std::sync::{Arc, Once};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use voicevox_core::{AccentPhraseModel, AudioQueryModel, TtsOptions, UserDictWord, VoiceModelId};
use voicevox_core::{StyleId, SupportedDevices, SynthesisOptions};

fn init_logger_once() {
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        let ansi = {
            // anstyle系のクレートを利用して次の2つを行う。
            //
            // * ANSI escape codeを出してよいかの判定（環境変数のチェックとisatty）
            // * 必要であれば`ENABLE_VIRTUAL_TERMINAL_PROCESSING`の有効化

            assert_eq!(
                ColorChoice::Auto,
                ColorChoice::global(),
                "`ColorChoice::write_global` should not have been called",
            );

            AutoStream::choice(&out()) != ColorChoice::Never
                && anstyle_query::term_supports_ansi_color()
                && anstyle_query::windows::enable_ansi_colors().unwrap_or(true)
        };

        // FIXME: `try_init` → `init` （subscriberは他に存在しないはずなので）
        let _ = tracing_subscriber::fmt()
            .with_env_filter(if env::var_os(EnvFilter::DEFAULT_ENV).is_some() {
                EnvFilter::from_default_env()
            } else {
                pub const ORT_LOGGING_LEVEL: &str = if cfg!(debug_assertions) {
                    "info"
                } else {
                    "warn"
                };
                format!("error,voicevox_core=info,voicevox_core_c_api=info,ort={ORT_LOGGING_LEVEL}")
                    .into()
            })
            .with_timer(local_time as fn(&mut Writer<'_>) -> _)
            .with_ansi(ansi)
            .with_writer(out)
            .try_init();
    });

    fn local_time(wtr: &mut Writer<'_>) -> fmt::Result {
        // ローカル時刻で表示はするが、そのフォーマットはtracing-subscriber本来のものに近いようにする。
        // https://github.com/tokio-rs/tracing/blob/tracing-subscriber-0.3.16/tracing-subscriber/src/fmt/time/datetime.rs#L235-L241
        wtr.write_str(&chrono::Local::now().to_rfc3339_opts(SecondsFormat::Micros, false))
    }

    fn out() -> impl RawStream {
        io::stderr()
    }
}

/*
 * Cの関数として公開するための型や関数を定義するこれらの実装はvoicevox_core/publish.rsに定義してある対応する関数にある
 * この関数ではvoicevox_core/publish.rsにある対応する関数の呼び出しと、その戻り値をCの形式に変換する処理のみとする
 * これはC文脈の処理と実装をわけるためと、内部実装の変更がAPIに影響を与えにくくするためである
 * voicevox_core/publish.rsにある対応する関数とはこのファイルに定義してある公開関数からvoicevoxプレフィックスを取り除いた名前の関数である
 */

/// テキスト解析器としてのOpen JTalk。
///
/// <b>構築</b>(_construction_)は ::voicevox_open_jtalk_rc_new で行い、<b>破棄</b>(_destruction_)は ::voicevox_open_jtalk_rc_delete で行う。
///
/// 参照カウント方式のスマートポインタ(reference-counted smart pointer)であり、
/// ::voicevox_synthesizer_new に渡されるときには参照カウンタがインクリメントされる形でオブジェクトの共有が行われる。
///
/// \example{
/// ```c
/// OpenJtalkRc *open_jtalk;
/// voicevox_open_jtalk_rc_new("./open_jtalk_dic_utf_8-1.11", &open_jtalk);
/// // ⋮
/// voicevox_open_jtalk_rc_delete(open_jtalk);
/// ```
/// }
pub struct OpenJtalkRc {
    open_jtalk: voicevox_core::blocking::OpenJtalk,
}

/// ::OpenJtalkRc を<b>構築</b>(_construct_)する。
///
/// 解放は ::voicevox_open_jtalk_rc_delete で行う。
///
/// @param [in] open_jtalk_dic_dir 辞書ディレクトリを指すUTF-8のパス
/// @param [out] out_open_jtalk 構築先
///
/// @returns 結果コード
///
/// \example{
/// ```c
/// OpenJtalkRc *open_jtalk;
/// voicevox_open_jtalk_rc_new("./open_jtalk_dic_utf_8-1.11", &open_jtalk);
/// ```
/// }
///
/// \safety{
/// - `open_jtalk_dic_dir`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `out_open_jtalk`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_open_jtalk_rc_new(
    open_jtalk_dic_dir: *const c_char,
    out_open_jtalk: NonNull<Box<OpenJtalkRc>>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let open_jtalk_dic_dir = ensure_utf8(CStr::from_ptr(open_jtalk_dic_dir))?;
        let open_jtalk = OpenJtalkRc::new(open_jtalk_dic_dir)?.into();
        out_open_jtalk.as_ptr().write_unaligned(open_jtalk);
        Ok(())
    })())
}

/// OpenJtalkの使うユーザー辞書を設定する。
///
/// この関数を呼び出した後にユーザー辞書を変更した場合、再度この関数を呼び出す必要がある。
///
/// @param [in] open_jtalk Open JTalkのオブジェクト
/// @param [in] user_dict ユーザー辞書
///
/// \safety{
/// - `open_jtalk`は ::voicevox_open_jtalk_rc_new で得たものでなければならず、また ::voicevox_open_jtalk_rc_delete で解放されていてはいけない。
/// - `user_dict`は ::voicevox_user_dict_new で得たものでなければならず、また ::voicevox_user_dict_delete で解放されていてはいけない。
/// }
#[no_mangle]
pub extern "C" fn voicevox_open_jtalk_rc_use_user_dict(
    open_jtalk: &OpenJtalkRc,
    user_dict: &VoicevoxUserDict,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        open_jtalk.open_jtalk.use_user_dict(&user_dict.dict)?;
        Ok(())
    })())
}

/// ::OpenJtalkRc を<b>破棄</b>(_destruct_)する。
///
/// @param [in] open_jtalk 破棄対象
///
/// \example{
/// ```c
/// voicevox_open_jtalk_rc_delete(open_jtalk);
/// ```
/// }
///
/// \safety{
/// - `open_jtalk`は ::voicevox_open_jtalk_rc_new で得たものでなければならず、また既にこの関数で解放されていてはいけない。
/// - `open_jtalk`は以後<b>ダングリングポインタ</b>(_dangling pointer_)として扱われなくてはならない。
/// }
#[no_mangle]
pub extern "C" fn voicevox_open_jtalk_rc_delete(open_jtalk: Box<OpenJtalkRc>) {
    init_logger_once();
    drop(open_jtalk);
}

/// ハードウェアアクセラレーションモードを設定する設定値。
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

/// ::voicevox_synthesizer_new のオプション。
#[repr(C)]
pub struct VoicevoxInitializeOptions {
    /// ハードウェアアクセラレーションモード
    acceleration_mode: VoicevoxAccelerationMode,
    /// CPU利用数を指定
    /// 0を指定すると環境に合わせたCPUが利用される
    cpu_num_threads: u16,
}

/// デフォルトの初期化オプションを生成する
/// @return デフォルト値が設定された初期化オプション
#[no_mangle]
pub extern "C" fn voicevox_make_default_initialize_options() -> VoicevoxInitializeOptions {
    init_logger_once();
    VoicevoxInitializeOptions::default()
}

/// voicevoxのバージョンを取得する。
/// @return SemVerでフォーマットされたバージョン。
#[no_mangle]
pub extern "C" fn voicevox_get_version() -> *const c_char {
    init_logger_once();
    return C_STRING_DROP_CHECKER.blacklist(VERSION).as_ptr();

    const VERSION: &CStr = unsafe {
        // SAFETY: The package version is a SemVer, so it should not contain '\0'
        CStr::from_bytes_with_nul_unchecked(concat!(env!("CARGO_PKG_VERSION"), '\0').as_bytes())
    };
}

/// 音声モデル。
///
/// VVMファイルと対応する。
/// <b>構築</b>(_construction_)は ::voicevox_voice_model_new_from_path で行い、<b>破棄</b>(_destruction_)は ::voicevox_voice_model_delete で行う。
#[derive(Getters)]
pub struct VoicevoxVoiceModel {
    model: voicevox_core::blocking::VoiceModel,
    id: CString,
    metas: CString,
}

/// 音声モデルID。
pub type VoicevoxVoiceModelId = *const c_char;

/// スタイルID。
///
/// VOICEVOXにおける、ある<b>話者</b>(_speaker_)のある<b>スタイル</b>(_style_)を指す。
pub type VoicevoxStyleId = u32;

/// VVMファイルから ::VoicevoxVoiceModel を<b>構築</b>(_construct_)する。
///
/// @param [in] path vvmファイルへのUTF-8のファイルパス
/// @param [out] out_model 構築先
///
/// @returns 結果コード
///
/// \safety{
/// - `path`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `out_model`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_voice_model_new_from_path(
    path: *const c_char,
    out_model: NonNull<Box<VoicevoxVoiceModel>>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let path = ensure_utf8(CStr::from_ptr(path))?;
        let model = VoicevoxVoiceModel::from_path(path)?.into();
        out_model.as_ptr().write_unaligned(model);
        Ok(())
    })())
}

/// ::VoicevoxVoiceModel からIDを取得する。
///
/// @param [in] model 音声モデル
///
/// @returns 音声モデルID
///
/// \safety{
/// - `model`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また ::voicevox_voice_model_delete で解放されていてはいけない。
/// }
#[no_mangle]
pub extern "C" fn voicevox_voice_model_id(model: &VoicevoxVoiceModel) -> VoicevoxVoiceModelId {
    init_logger_once();
    model.id().as_ptr()
}

/// ::VoicevoxVoiceModel からメタ情報を取得する。
///
/// @param [in] model 音声モデル
///
/// @returns メタ情報のJSON文字列
///
/// \safety{
/// - `model`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また ::voicevox_voice_model_delete で解放されていてはいけない。
/// - 戻り値の文字列の<b>生存期間</b>(_lifetime_)は次にこの関数が呼ばれるか、`model`が破棄されるまでである。この生存期間を越えて文字列にアクセスしてはならない。
/// }
#[no_mangle]
pub extern "C" fn voicevox_voice_model_get_metas_json(model: &VoicevoxVoiceModel) -> *const c_char {
    init_logger_once();
    model.metas().as_ptr()
}

/// ::VoicevoxVoiceModel を<b>破棄</b>(_destruct_)する。
///
/// @param [in] model 破棄対象
///
/// \safety{
/// - `model`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また既にこの関数で解放されていてはいけない。
/// - `model`は以後<b>ダングリングポインタ</b>(_dangling pointer_)として扱われなくてはならない。
/// }
#[no_mangle]
pub extern "C" fn voicevox_voice_model_delete(model: Box<VoicevoxVoiceModel>) {
    init_logger_once();
    drop(model);
}

/// 音声シンセサイザ。
///
/// <b>構築</b>(_construction_)は ::voicevox_synthesizer_new で行い、<b>破棄</b>(_destruction_)は ::voicevox_synthesizer_delete で行う。
#[derive(Getters)]
pub struct VoicevoxSynthesizer {
    synthesizer: voicevox_core::blocking::Synthesizer<voicevox_core::blocking::OpenJtalk>,
}

/// ::VoicevoxSynthesizer を<b>構築</b>(_construct_)する。
///
/// @param [in] open_jtalk Open JTalkのオブジェクト
/// @param [in] options オプション
/// @param [out] out_synthesizer 構築先
///
/// @returns 結果コード
///
/// \safety{
/// - `open_jtalk`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また ::voicevox_open_jtalk_rc_new で解放されていてはいけない。
/// - `out_synthesizer`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_new(
    open_jtalk: &OpenJtalkRc,
    options: VoicevoxInitializeOptions,
    out_synthesizer: NonNull<Box<VoicevoxSynthesizer>>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let options = options.into();

        let synthesizer = VoicevoxSynthesizer::new(open_jtalk, &options)?.into();
        out_synthesizer.as_ptr().write_unaligned(synthesizer);
        Ok(())
    })())
}

/// ::VoicevoxSynthesizer を<b>破棄</b>(_destruct_)する。
///
/// @param [in] synthesizer 破棄対象
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また既にこの関数で解放されていてはいけない。
/// - `synthesizer`は以後<b>ダングリングポインタ</b>(_dangling pointer_)として扱われなくてはならない。
/// }
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_delete(synthesizer: Box<VoicevoxSynthesizer>) {
    init_logger_once();
    drop(synthesizer);
}

/// 音声モデルを読み込む。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] model 音声モデル
///
/// @returns 結果コード
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `model`は ::voicevox_voice_model_new_from_path で得たものでなければならず、また ::voicevox_voice_model_delete で解放されていてはいけない。
/// }
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_load_voice_model(
    synthesizer: &VoicevoxSynthesizer,
    model: &VoicevoxVoiceModel,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error(synthesizer.load_voice_model(model.model()))
}

/// 音声モデルの読み込みを解除する。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] model_id 音声モデルID
///
/// @returns 結果コード
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `model_id`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_unload_voice_model(
    synthesizer: &VoicevoxSynthesizer,
    model_id: VoicevoxVoiceModelId,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let raw_model_id = ensure_utf8(unsafe { CStr::from_ptr(model_id) })?;
        synthesizer
            .unload_voice_model(&VoiceModelId::new(raw_model_id.to_string()))
            .map_err(Into::into)
    })())
}

/// ハードウェアアクセラレーションがGPUモードか判定する。
///
/// @param [in] synthesizer 音声シンセサイザ
///
/// @returns GPUモードかどうか
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// }
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_is_gpu_mode(synthesizer: &VoicevoxSynthesizer) -> bool {
    init_logger_once();
    synthesizer.synthesizer().is_gpu_mode()
}

/// 指定したIDの音声モデルが読み込まれているか判定する。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] model_id 音声モデルID
///
/// @returns モデルが読み込まれているかどうか
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `model_id`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_is_loaded_voice_model(
    synthesizer: &VoicevoxSynthesizer,
    model_id: VoicevoxVoiceModelId,
) -> bool {
    init_logger_once();
    let Ok(raw_model_id) = ensure_utf8(unsafe { CStr::from_ptr(model_id) }) else {
        // 与えられたIDがUTF-8ではない場合、それに対応する`VoicdModel`は確実に存在しない
        return false;
    };
    synthesizer
        .synthesizer()
        .is_loaded_voice_model(&VoiceModelId::new(raw_model_id.into()))
}

/// 今読み込んでいる音声モデルのメタ情報を、JSONで取得する。
///
/// JSONの解放は ::voicevox_json_free で行う。
///
/// @param [in] synthesizer 音声シンセサイザ
///
/// @return メタ情報のJSON文字列
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// }
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_create_metas_json(
    synthesizer: &VoicevoxSynthesizer,
) -> *mut c_char {
    init_logger_once();
    let metas = synthesizer.metas();
    C_STRING_DROP_CHECKER.whitelist(metas).into_raw()
}

/// このライブラリで利用可能なデバイスの情報を、JSONで取得する。
///
/// JSONの解放は ::voicevox_json_free で行う。
///
/// あくまで本ライブラリが対応しているデバイスの情報であることに注意。GPUが使える環境ではなかったとしても`cuda`や`dml`は`true`を示しうる。
///
/// @param [out] output_supported_devices_json サポートデバイス情報のJSON文字列
///
/// @returns 結果コード
///
/// \example{
/// ```c
/// char *supported_devices;
/// VoicevoxResultCode result = voicevox_create_supported_devices_json(&supported_devices);
/// ```
/// }
///
/// \safety{
/// - `output_supported_devices_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_create_supported_devices_json(
    output_supported_devices_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
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

/// AquesTalk風記法から、AudioQueryをJSONとして生成する。
///
/// 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] kana AquesTalk風記法
/// @param [in] style_id スタイルID
/// @param [out] output_audio_query_json 生成先
///
/// @returns 結果コード
///
/// \example{
/// ```c
/// char *audio_query;
/// voicevox_synthesizer_create_audio_query_from_kana(synthesizer, "コンニチワ'",
///                                                   2, // "四国めたん (ノーマル)"
///                                                   &audio_query);
/// ```
/// }
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `kana`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_create_audio_query_from_kana(
    synthesizer: &VoicevoxSynthesizer,
    kana: *const c_char,
    style_id: VoicevoxStyleId,
    output_audio_query_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let kana = CStr::from_ptr(kana);
        let kana = ensure_utf8(kana)?;

        let audio_query = synthesizer
            .synthesizer()
            .audio_query_from_kana(kana, StyleId::new(style_id))?;
        let audio_query = CString::new(audio_query_model_to_json(&audio_query))
            .expect("should not contain '\\0'");
        output_audio_query_json
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(audio_query).into_raw());
        Ok(())
    })())
}

/// 日本語テキストから、AudioQueryをJSONとして生成する。
///
/// 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] text UTF-8の日本語テキスト
/// @param [in] style_id スタイルID
/// @param [out] output_audio_query_json 生成先
///
/// @returns 結果コード
///
/// \example{
/// ```c
/// char *audio_query;
/// voicevox_synthesizer_create_audio_query(synthesizer, "こんにちは",
///                                         2, // "四国めたん (ノーマル)"
///                                         &audio_query);
/// ```
/// }
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `text`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_create_audio_query(
    synthesizer: &VoicevoxSynthesizer,
    text: *const c_char,
    style_id: VoicevoxStyleId,
    output_audio_query_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let text = CStr::from_ptr(text);
        let text = ensure_utf8(text)?;

        let audio_query = synthesizer
            .synthesizer()
            .audio_query(text, StyleId::new(style_id))?;
        let audio_query = CString::new(audio_query_model_to_json(&audio_query))
            .expect("should not contain '\\0'");
        output_audio_query_json
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(audio_query).into_raw());
        Ok(())
    })())
}

/// AquesTalk風記法から、AccentPhrase (アクセント句)の配列をJSON形式で生成する。
///
/// 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] kana AquesTalk風記法
/// @param [in] style_id スタイルID
/// @param [out] output_accent_phrases_json 生成先
///
/// @returns 結果コード
///
/// \example{
/// ```c
/// char *accent_phrases;
/// voicevox_synthesizer_create_accent_phrases_from_kana(
///     synthesizer, "コンニチワ'",
///     2, // "四国めたん (ノーマル)"
///     &accent_phrases);
/// ```
/// }
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `kana`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_create_accent_phrases_from_kana(
    synthesizer: &VoicevoxSynthesizer,
    kana: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let kana = ensure_utf8(CStr::from_ptr(kana))?;
        let accent_phrases = synthesizer
            .synthesizer()
            .create_accent_phrases_from_kana(kana, StyleId::new(style_id))?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

/// 日本語テキストから、AccentPhrase (アクセント句)の配列をJSON形式で生成する。
///
/// 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] text UTF-8の日本語テキスト
/// @param [in] style_id スタイルID
/// @param [out] output_accent_phrases_json 生成先
///
/// @returns 結果コード
///
/// \example{
/// ```c
/// char *accent_phrases;
/// voicevox_synthesizer_create_accent_phrases(synthesizer, "こんにちは",
///                                            2, // "四国めたん (ノーマル)"
///                                            &accent_phrases);
/// ```
/// }
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `text`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_create_accent_phrases(
    synthesizer: &VoicevoxSynthesizer,
    text: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let text = ensure_utf8(CStr::from_ptr(text))?;
        let accent_phrases = synthesizer
            .synthesizer()
            .create_accent_phrases(text, StyleId::new(style_id))?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

/// AccentPhraseの配列の音高・音素長を、特定の声で生成しなおす。
///
/// 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] accent_phrases_json AccentPhraseの配列のJSON文字列
/// @param [in] style_id スタイルID
/// @param [out] output_accent_phrases_json 生成先
///
/// @returns 結果コード
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `accent_phrases_json`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_replace_mora_data(
    synthesizer: &VoicevoxSynthesizer,
    accent_phrases_json: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let accent_phrases: Vec<AccentPhraseModel> =
            serde_json::from_str(ensure_utf8(CStr::from_ptr(accent_phrases_json))?)
                .map_err(CApiError::InvalidAccentPhrase)?;
        let accent_phrases = synthesizer
            .synthesizer()
            .replace_mora_data(&accent_phrases, StyleId::new(style_id))?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

/// AccentPhraseの配列の音素長を、特定の声で生成しなおす。
///
/// 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] accent_phrases_json AccentPhraseの配列のJSON文字列
/// @param [in] style_id スタイルID
/// @param [out] output_accent_phrases_json 生成先
///
/// @returns 結果コード
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `accent_phrases_json`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_replace_phoneme_length(
    synthesizer: &VoicevoxSynthesizer,
    accent_phrases_json: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let accent_phrases: Vec<AccentPhraseModel> =
            serde_json::from_str(ensure_utf8(CStr::from_ptr(accent_phrases_json))?)
                .map_err(CApiError::InvalidAccentPhrase)?;
        let accent_phrases = synthesizer
            .synthesizer()
            .replace_phoneme_length(&accent_phrases, StyleId::new(style_id))?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

/// AccentPhraseの配列の音高を、特定の声で生成しなおす。
///
/// 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] accent_phrases_json AccentPhraseの配列のJSON文字列
/// @param [in] style_id スタイルID
/// @param [out] output_accent_phrases_json 生成先
///
/// @returns 結果コード
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `accent_phrases_json`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_replace_mora_pitch(
    synthesizer: &VoicevoxSynthesizer,
    accent_phrases_json: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let accent_phrases: Vec<AccentPhraseModel> =
            serde_json::from_str(ensure_utf8(CStr::from_ptr(accent_phrases_json))?)
                .map_err(CApiError::InvalidAccentPhrase)?;
        let accent_phrases = synthesizer
            .synthesizer()
            .replace_mora_pitch(&accent_phrases, StyleId::new(style_id))?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .as_ptr()
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

/// ::voicevox_synthesizer_synthesis のオプション。
#[repr(C)]
pub struct VoicevoxSynthesisOptions {
    /// 疑問文の調整を有効にする
    enable_interrogative_upspeak: bool,
}

/// デフォルトの `voicevox_synthesizer_synthesis` のオプションを生成する
/// @return デフォルト値が設定された `voicevox_synthesizer_synthesis` のオプション
#[no_mangle]
pub extern "C" fn voicevox_make_default_synthesis_options() -> VoicevoxSynthesisOptions {
    init_logger_once();
    VoicevoxSynthesisOptions::default()
}

/// AudioQueryから音声合成を行う。
///
/// 生成したWAVデータを解放するには ::voicevox_wav_free を使う。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] audio_query_json AudioQueryのJSON文字列
/// @param [in] style_id スタイルID
/// @param [in] options オプション
/// @param [out] output_wav_length 出力のバイト長
/// @param [out] output_wav 出力先
///
/// @returns 結果コード
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `audio_query_json`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_wav_length`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// - `output_wav`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_synthesis(
    synthesizer: &VoicevoxSynthesizer,
    audio_query_json: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxSynthesisOptions,
    output_wav_length: NonNull<usize>,
    output_wav: NonNull<*mut u8>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let audio_query_json = CStr::from_ptr(audio_query_json)
            .to_str()
            .map_err(|_| CApiError::InvalidUtf8Input)?;
        let audio_query: AudioQueryModel =
            serde_json::from_str(audio_query_json).map_err(CApiError::InvalidAudioQuery)?;
        let wav = synthesizer.synthesizer().synthesis(
            &audio_query,
            StyleId::new(style_id),
            &SynthesisOptions::from(options),
        )?;
        U8_SLICE_OWNER.own_and_lend(wav, output_wav, output_wav_length);
        Ok(())
    })())
}

/// ::voicevox_synthesizer_tts のオプション。
#[repr(C)]
pub struct VoicevoxTtsOptions {
    /// 疑問文の調整を有効にする
    enable_interrogative_upspeak: bool,
}

/// デフォルトのテキスト音声合成オプションを生成する
/// @return テキスト音声合成オプション
#[no_mangle]
pub extern "C" fn voicevox_make_default_tts_options() -> VoicevoxTtsOptions {
    init_logger_once();
    voicevox_core::TtsOptions::default().into()
}

/// AquesTalk風記法から音声合成を行う。
///
/// 生成したWAVデータを解放するには ::voicevox_wav_free を使う。
///
/// @param [in] synthesizer
/// @param [in] kana AquesTalk風記法
/// @param [in] style_id スタイルID
/// @param [in] options オプション
/// @param [out] output_wav_length 出力のバイト長
/// @param [out] output_wav 出力先
///
/// @returns 結果コード
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `kana`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_wav_length`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// - `output_wav`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_tts_from_kana(
    synthesizer: &VoicevoxSynthesizer,
    kana: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxTtsOptions,
    output_wav_length: NonNull<usize>,
    output_wav: NonNull<*mut u8>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let kana = ensure_utf8(CStr::from_ptr(kana))?;
        let output = synthesizer.synthesizer().tts_from_kana(
            kana,
            StyleId::new(style_id),
            &TtsOptions::from(options),
        )?;
        U8_SLICE_OWNER.own_and_lend(output, output_wav, output_wav_length);
        Ok(())
    })())
}

/// 日本語テキストから音声合成を行う。
///
/// 生成したWAVデータを解放するには ::voicevox_wav_free を使う。
///
/// @param [in] synthesizer
/// @param [in] text UTF-8の日本語テキスト
/// @param [in] style_id スタイルID
/// @param [in] options オプション
/// @param [out] output_wav_length 出力のバイト長
/// @param [out] output_wav 出力先
///
/// @returns 結果コード
///
/// \safety{
/// - `synthesizer`は ::voicevox_synthesizer_new で得たものでなければならず、また ::voicevox_synthesizer_delete で解放されていてはいけない。
/// - `text`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_wav_length`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// - `output_wav`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_tts(
    synthesizer: &VoicevoxSynthesizer,
    text: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxTtsOptions,
    output_wav_length: NonNull<usize>,
    output_wav: NonNull<*mut u8>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let text = ensure_utf8(CStr::from_ptr(text))?;
        let output = synthesizer.synthesizer().tts(
            text,
            StyleId::new(style_id),
            &TtsOptions::from(options),
        )?;
        U8_SLICE_OWNER.own_and_lend(output, output_wav, output_wav_length);
        Ok(())
    })())
}

/// JSON文字列を解放する。
///
/// @param [in] json 解放するJSON文字列
///
/// \safety{
/// - `json`は以下のAPIで得られたポインタでなくてはいけない。
///     - ::voicevox_create_supported_devices_json
///     - ::voicevox_synthesizer_create_metas_json
///     - ::voicevox_synthesizer_create_audio_query
///     - ::voicevox_synthesizer_create_accent_phrases
///     - ::voicevox_synthesizer_replace_mora_data
///     - ::voicevox_synthesizer_replace_phoneme_length
///     - ::voicevox_synthesizer_replace_mora_pitch
///     - ::voicevox_user_dict_to_json
/// - 文字列の長さは生成時より変更されていてはならない。
/// - `json`は<a href="#voicevox-core-safety">読み込みと書き込みについて有効</a>でなければならない。
/// - `json`は以後<b>ダングリングポインタ</b>(_dangling pointer_)として扱われなくてはならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_json_free(json: *mut c_char) {
    init_logger_once();
    drop(CString::from_raw(C_STRING_DROP_CHECKER.check(json)));
}

/// WAVデータを解放する。
///
/// @param [in] wav 解放するWAVデータ
///
/// \safety{
/// - `wav`は以下のAPIで得られたポインタでなくてはいけない。
///     - ::voicevox_synthesizer_synthesis
///     - ::voicevox_synthesizer_tts
/// - `wav`は<a href="#voicevox-core-safety">読み込みと書き込みについて有効</a>でなければならない。
/// - `wav`は以後<b>ダングリングポインタ</b>(_dangling pointer_)として扱われなくてはならない。
/// }
#[no_mangle]
pub extern "C" fn voicevox_wav_free(wav: *mut u8) {
    init_logger_once();
    U8_SLICE_OWNER.drop_for(wav);
}

/// 結果コードに対応したメッセージ文字列を取得する。
///
/// @param [in] result_code 結果コード
///
/// @returns 結果コードに対応したメッセージ文字列
///
/// \examples{
/// ```c
/// const char *actual = voicevox_error_result_to_message(VOICEVOX_RESULT_OK);
/// const char *EXPECTED = "エラーが発生しませんでした";
/// assert(strcmp(actual, EXPECTED) == 0);
/// ```
///
/// ```c
/// const char *actual =
///     voicevox_error_result_to_message(VOICEVOX_RESULT_LOAD_MODEL_ERROR);
/// const char *EXPECTED = "modelデータ読み込みに失敗しました";
/// assert(strcmp(actual, EXPECTED) == 0);
/// ```
/// }
#[no_mangle]
pub extern "C" fn voicevox_error_result_to_message(
    result_code: VoicevoxResultCode,
) -> *const c_char {
    init_logger_once();
    let message = result_code::error_result_to_message(result_code);
    C_STRING_DROP_CHECKER.blacklist(message).as_ptr()
}

/// ユーザー辞書。
#[derive(Default)]
pub struct VoicevoxUserDict {
    dict: Arc<voicevox_core::blocking::UserDict>,
}

/// ユーザー辞書の単語。
#[derive(Clone, Copy)]
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

/// ユーザー辞書の単語の種類。
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

/// ::VoicevoxUserDictWord を最低限のパラメータで作成する。
///
/// @param [in] surface 表記
/// @param [in] pronunciation 読み
/// @returns ::VoicevoxUserDictWord
#[no_mangle]
pub extern "C" fn voicevox_user_dict_word_make(
    surface: *const c_char,
    pronunciation: *const c_char,
) -> VoicevoxUserDictWord {
    init_logger_once();
    VoicevoxUserDictWord {
        surface,
        pronunciation,
        accent_type: UserDictWord::default().accent_type,
        word_type: UserDictWord::default().word_type.into(),
        priority: UserDictWord::default().priority,
    }
}

/// ユーザー辞書をb>構築</b>(_construct_)する。
///
/// @returns ::VoicevoxUserDict
#[no_mangle]
pub extern "C" fn voicevox_user_dict_new() -> Box<VoicevoxUserDict> {
    init_logger_once();
    Default::default()
}

/// ユーザー辞書にファイルを読み込ませる。
///
/// @param [in] user_dict ユーザー辞書
/// @param [in] dict_path 読み込む辞書ファイルのパス
/// @returns 結果コード
///
/// \safety{
/// - `user_dict`は ::voicevox_user_dict_new で得たものでなければならず、また ::voicevox_user_dict_delete で解放されていてはいけない。
/// - `dict_path`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_load(
    user_dict: &VoicevoxUserDict,
    dict_path: *const c_char,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let dict_path = ensure_utf8(unsafe { CStr::from_ptr(dict_path) })?;
        user_dict.dict.load(dict_path)?;

        Ok(())
    })())
}

/// ユーザー辞書に単語を追加する。
///
/// @param [in] ユーザー辞書
/// @param [in] word 追加する単語
/// @param [out] output_word_uuid 追加した単語のUUID
/// @returns 結果コード
///
/// # Safety
/// @param user_dict は有効な :VoicevoxUserDict のポインタであること
///
/// \safety{
/// - `user_dict`は ::voicevox_user_dict_new で得たものでなければならず、また ::voicevox_user_dict_delete で解放されていてはいけない。
/// - `word->surface`と`word->pronunciation`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_word_uuid`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_add_word(
    user_dict: &VoicevoxUserDict,
    word: *const VoicevoxUserDictWord,
    output_word_uuid: NonNull<[u8; 16]>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let word = word.read_unaligned().try_into_word()?;
        let uuid = user_dict.dict.add_word(word)?;
        output_word_uuid.as_ptr().copy_from(uuid.as_bytes(), 16);

        Ok(())
    })())
}

/// ユーザー辞書の単語を更新する。
///
/// @param [in] user_dict ユーザー辞書
/// @param [in] word_uuid 更新する単語のUUID
/// @param [in] word 新しい単語のデータ
/// @returns 結果コード
///
/// \safety{
/// - `user_dict`は ::voicevox_user_dict_new で得たものでなければならず、また ::voicevox_user_dict_delete で解放されていてはいけない。
/// - `word_uuid`は<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `word->surface`と`word->pronunciation`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_update_word(
    user_dict: &VoicevoxUserDict,
    word_uuid: &[u8; 16],
    word: *const VoicevoxUserDictWord,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let word_uuid = Uuid::from_slice(word_uuid).map_err(CApiError::InvalidUuid)?;
        let word = word.read_unaligned().try_into_word()?;
        user_dict.dict.update_word(word_uuid, word)?;

        Ok(())
    })())
}

/// ユーザー辞書から単語を削除する。
///
/// @param [in] user_dict ユーザー辞書
/// @param [in] word_uuid 削除する単語のUUID
/// @returns 結果コード
///
/// \safety{
/// - `user_dict`は ::voicevox_user_dict_new で得たものでなければならず、また ::voicevox_user_dict_delete で解放されていてはいけない。
/// - `word_uuid`は<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub extern "C" fn voicevox_user_dict_remove_word(
    user_dict: &VoicevoxUserDict,
    word_uuid: &[u8; 16],
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let word_uuid = Uuid::from_slice(word_uuid).map_err(CApiError::InvalidUuid)?;
        user_dict.dict.remove_word(word_uuid)?;
        Ok(())
    })())
}

// FIXME: infallibleなので、`char*`を戻り値にしてもよいはず
/// ユーザー辞書の単語をJSON形式で出力する。
///
/// 生成したJSON文字列を解放するには ::voicevox_json_free を使う。
///
/// @param [in] user_dict ユーザー辞書
/// @param [out] output_json 出力先
/// @returns 結果コード
///
/// \safety{
/// - `user_dict`は ::voicevox_user_dict_new で得たものでなければならず、また ::voicevox_user_dict_delete で解放されていてはいけない。
/// - `output_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_to_json(
    user_dict: &VoicevoxUserDict,
    output_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    let json = user_dict.dict.to_json();
    let json = CString::new(json).expect("\\0を含まない文字列であることが保証されている");
    output_json
        .as_ptr()
        .write_unaligned(C_STRING_DROP_CHECKER.whitelist(json).into_raw());
    VoicevoxResultCode::VOICEVOX_RESULT_OK
}

/// 他のユーザー辞書をインポートする。
///
/// @param [in] user_dict ユーザー辞書
/// @param [in] other_dict インポートするユーザー辞書
/// @returns 結果コード
///
/// \safety{
/// - `user_dict`と`other_dict`は ::voicevox_user_dict_new で得たものでなければならず、また ::voicevox_user_dict_delete で解放されていてはいけない。
/// }
#[no_mangle]
pub extern "C" fn voicevox_user_dict_import(
    user_dict: &VoicevoxUserDict,
    other_dict: &VoicevoxUserDict,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        user_dict.dict.import(&other_dict.dict)?;
        Ok(())
    })())
}

/// ユーザー辞書をファイルに保存する。
///
/// @param [in] user_dict ユーザー辞書
/// @param [in] path 保存先のファイルパス
///
/// \safety{
/// - `user_dict`は ::voicevox_user_dict_new で得たものでなければならず、また ::voicevox_user_dict_delete で解放されていてはいけない。
/// - `path`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_save(
    user_dict: &VoicevoxUserDict,
    path: *const c_char,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let path = ensure_utf8(CStr::from_ptr(path))?;
        user_dict.dict.save(path)?;
        Ok(())
    })())
}

/// ユーザー辞書を<b>破棄</b>(_destruct_)する。
///
/// @param [in] user_dict 破棄対象
///
/// \safety{
/// - `user_dict`は ::voicevox_user_dict_new で得たものでなければならず、また既にこの関数で解放されていてはいけない。
/// }
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_delete(user_dict: Box<VoicevoxUserDict>) {
    init_logger_once();
    drop(user_dict);
}
