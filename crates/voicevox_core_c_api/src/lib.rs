// ここにある`#[doc]`はすべてrustdocではなくDoxygen向けのものである。
#![allow(
    clippy::doc_lazy_continuation,
    clippy::missing_safety_doc // safety documentation自体は書くが、Doxygenの慣習に従い`<dt>`で書く
)]

mod c_impls;
/// cbindgen:ignore
#[cfg(feature = "load-onnxruntime")]
mod compatible_engine;
mod drop_check;
mod helpers;
mod object;
mod result_code;
mod slice_owner;
use self::drop_check::C_STRING_DROP_CHECKER;
use self::helpers::{
    accent_phrases_to_json, audio_query_model_to_json, ensure_utf8, into_result_code_with_error,
    CApiError, UuidBytesExt as _,
};
use self::object::{CApiObject as _, CApiObjectPtrExt as _};
use self::result_code::VoicevoxResultCode;
use self::slice_owner::U8_SLICE_OWNER;
use anstream::{stream::RawStream, AutoStream};
use c_impls::{VoicevoxSynthesizerPtrExt as _, VoicevoxVoiceModelFilePtrExt as _};
use chrono::SecondsFormat;
use colorchoice::ColorChoice;
use educe::Educe;
use ref_cast::RefCastCustom;
use std::env;
use std::ffi::{CStr, CString};
use std::fmt;
use std::io;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use std::ptr::NonNull;
use std::sync::Once;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;
use voicevox_core::__internal::interop::{ToJsonValue as _, DEFAULT_PRIORITY, DEFAULT_WORD_TYPE};
use voicevox_core::{AccentPhrase, AudioQuery, StyleId};

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

        tracing_subscriber::fmt()
            .with_env_filter(if env::var_os(EnvFilter::DEFAULT_ENV).is_some() {
                EnvFilter::from_default_env()
            } else {
                "error,voicevox_core=info,voicevox_core_c_api=info,ort=warn".into()
            })
            .with_timer(local_time as fn(&mut Writer<'_>) -> _)
            .with_ansi(ansi)
            .with_writer(out)
            .init();
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

// TODO: https://github.com/mozilla/cbindgen/issues/927
//#[cfg(feature = "load-onnxruntime")]
//pub const VOICEVOX_ONNXRUNTIME_LIB_NAME: &CStr = ..;
//#[cfg(feature = "load-onnxruntime")]
//pub const VOICEVOX_ONNXRUNTIME_LIB_VERSION: &CStr = ..;

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ONNX Runtimeの動的ライブラリの、バージョン付きのファイル名。
///
/// WindowsとAndroidでは ::voicevox_get_onnxruntime_lib_unversioned_filename と同じ。
///
/// \availability{
///   [リリース](https://github.com/voicevox/voicevox_core/releases)されているライブラリではiOSを除くプラットフォームで利用可能。詳細は<a href="#voicevox-core-availability">ファイルレベルの"Availability"の節</a>を参照。
/// }
///
/// \orig-impl{voicevox_get_onnxruntime_lib_versioned_filename}
#[cfg(feature = "load-onnxruntime")]
#[no_mangle]
pub extern "C" fn voicevox_get_onnxruntime_lib_versioned_filename() -> *const c_char {
    init_logger_once();
    const FILENAME: &CStr = VoicevoxOnnxruntime::LIB_VERSIONED_FILENAME;
    C_STRING_DROP_CHECKER.blacklist(FILENAME).as_ptr()
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ONNX Runtimeの動的ライブラリの、バージョン無しのファイル名。
///
/// \availability{
///   [リリース](https://github.com/voicevox/voicevox_core/releases)されているライブラリではiOSを除くプラットフォームで利用可能。詳細は<a href="#voicevox-core-availability">ファイルレベルの"Availability"の節</a>を参照。
/// }
///
/// \orig-impl{voicevox_get_onnxruntime_lib_unversioned_filename}
#[cfg(feature = "load-onnxruntime")]
#[no_mangle]
pub extern "C" fn voicevox_get_onnxruntime_lib_unversioned_filename() -> *const c_char {
    init_logger_once();
    const FILENAME: &CStr = VoicevoxOnnxruntime::LIB_UNVERSIONED_FILENAME;
    C_STRING_DROP_CHECKER.blacklist(FILENAME).as_ptr()
}

/// ::voicevox_onnxruntime_load_once のオプション。
///
/// \availability{
///   [リリース](https://github.com/voicevox/voicevox_core/releases)されているライブラリではiOSを除くプラットフォームで利用可能。詳細は<a href="#voicevox-core-availability">ファイルレベルの"Availability"の節</a>を参照。
/// }
///
/// \no-orig-impl{VoicevoxLoadOnnxruntimeOptions}
#[cfg(feature = "load-onnxruntime")]
#[repr(C)]
pub struct VoicevoxLoadOnnxruntimeOptions {
    /// ONNX Runtimeのファイル名（モジュール名）もしくはファイルパスを指定する。
    ///
    /// `dlopen`/[`LoadLibraryExW`](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw)の引数に使われる。デフォルトは ::voicevox_get_onnxruntime_lib_versioned_filename と同じ。
    filename: *const c_char,
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// デフォルトの ::voicevox_onnxruntime_load_once のオプションを生成する。
///
/// @return デフォルトの ::voicevox_onnxruntime_load_once のオプション
///
/// \availability{
///   [リリース](https://github.com/voicevox/voicevox_core/releases)されているライブラリではiOSを除くプラットフォームで利用可能。詳細は<a href="#voicevox-core-availability">ファイルレベルの"Availability"の節</a>を参照。
/// }
///
/// \no-orig-impl{voicevox_make_default_load_onnxruntime_options}
#[cfg(feature = "load-onnxruntime")]
#[no_mangle]
pub extern "C" fn voicevox_make_default_load_onnxruntime_options() -> VoicevoxLoadOnnxruntimeOptions
{
    init_logger_once();
    let filename = VoicevoxOnnxruntime::LIB_VERSIONED_FILENAME;
    let filename = C_STRING_DROP_CHECKER.blacklist(filename).as_ptr();
    VoicevoxLoadOnnxruntimeOptions { filename }
}

// https://github.com/mozilla/cbindgen/issues/967
/// ONNX Runtime。
///
/// シングルトンであり、インスタンスは高々一つ。
///
/// ```c
/// const VoicevoxOnnxruntime *ort1;
/// voicevox_onnxruntime_load_once(
///     voicevox_make_default_load_onnxruntime_options(), &ort1);
/// const VoicevoxOnnxruntime *ort2 = voicevox_onnxruntime_get();
/// assert(ort1 == ort2);
/// ```
///
/// \orig-impl{VoicevoxOnnxruntime}
#[cfg(any())]
pub struct VoicevoxOnnxruntime(!);

/// cbindgen:ignore
#[derive(RefCastCustom)]
#[repr(transparent)]
pub struct VoicevoxOnnxruntime(voicevox_core::blocking::Onnxruntime);

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ::VoicevoxOnnxruntime のインスタンスが既に作られているならそれを得る。
///
/// 作られていなければ`NULL`を返す。
///
/// @returns ::VoicevoxOnnxruntime のインスタンス
///
/// \orig-impl{voicevox_onnxruntime_get}
#[no_mangle]
pub extern "C" fn voicevox_onnxruntime_get() -> Option<&'static VoicevoxOnnxruntime> {
    VoicevoxOnnxruntime::get()
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ONNX Runtimeをロードして初期化する。
///
/// 一度成功したら、以後は引数を無視して同じ参照を返す。
///
/// @param [in] options オプション
/// @param [out] out_onnxruntime ::VoicevoxOnnxruntime のインスタンス
///
/// @returns 結果コード
///
/// \availability{
///   [リリース](https://github.com/voicevox/voicevox_core/releases)されているライブラリではiOSを除くプラットフォームで利用可能。詳細は<a href="#voicevox-core-availability">ファイルレベルの"Availability"の節</a>を参照。
/// }
///
/// \safety{
/// - `options.filename`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `out_onnxruntime`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_onnxruntime_load_once}
#[cfg(feature = "load-onnxruntime")]
#[no_mangle]
pub unsafe extern "C" fn voicevox_onnxruntime_load_once(
    options: VoicevoxLoadOnnxruntimeOptions,
    out_onnxruntime: NonNull<&'static VoicevoxOnnxruntime>,
) -> VoicevoxResultCode {
    init_logger_once();
    let filename = unsafe {
        // SAFETY: ユーザーに要求している条件で十分
        CStr::from_ptr(options.filename)
    };
    into_result_code_with_error((|| {
        let instance = VoicevoxOnnxruntime::load_once(filename)?;
        unsafe {
            // SAFETY: ユーザーに要求している条件で十分
            out_onnxruntime.write_unaligned(instance);
        }
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ONNX Runtimeを初期化する。
///
/// 一度成功したら以後は同じ参照を返す。
///
/// @param [out] out_onnxruntime ::VoicevoxOnnxruntime のインスタンス
///
/// @returns 結果コード
///
/// \availability{
///   [リリース](https://github.com/voicevox/voicevox_core/releases)されているライブラリではiOSでのみ利用可能。詳細は<a href="#voicevox-core-availability">ファイルレベルの"Availability"の節</a>を参照。
/// }
///
/// \safety{
/// - `out_onnxruntime`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_onnxruntime_init_once}
#[cfg(feature = "link-onnxruntime")]
#[no_mangle]
pub unsafe extern "C" fn voicevox_onnxruntime_init_once(
    out_onnxruntime: NonNull<&'static VoicevoxOnnxruntime>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let instance = VoicevoxOnnxruntime::init_once()?;
        unsafe {
            // SAFETY: ユーザーに要求している条件で十分
            out_onnxruntime.write_unaligned(instance);
        }
        Ok(())
    })())
}

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
///
/// \orig-impl{OpenJtalkRc}
#[derive(Debug, Educe)]
#[educe(Default(expression = "Self { _padding: MaybeUninit::uninit() }"))]
pub struct OpenJtalkRc {
    _padding: MaybeUninit<[u8; 1]>,
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
///
/// \orig-impl{voicevox_open_jtalk_rc_new}
#[no_mangle]
pub unsafe extern "C" fn voicevox_open_jtalk_rc_new(
    open_jtalk_dic_dir: *const c_char,
    out_open_jtalk: NonNull<NonNull<OpenJtalkRc>>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let open_jtalk_dic_dir = ensure_utf8(CStr::from_ptr(open_jtalk_dic_dir))?;
        let open_jtalk = OpenJtalkRc::new(open_jtalk_dic_dir)?;
        out_open_jtalk.write_unaligned(open_jtalk);
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// OpenJtalkの使うユーザー辞書を設定する。
///
/// この関数を呼び出した後にユーザー辞書を変更した場合、再度この関数を呼び出す必要がある。
///
/// @param [in] open_jtalk Open JTalkのオブジェクト
/// @param [in] user_dict ユーザー辞書
///
/// \orig-impl{voicevox_open_jtalk_rc_use_user_dict}
#[no_mangle]
pub extern "C" fn voicevox_open_jtalk_rc_use_user_dict(
    open_jtalk: *const OpenJtalkRc,
    user_dict: *const VoicevoxUserDict,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        open_jtalk.body().use_user_dict(&user_dict.body())?;
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ::OpenJtalkRc を<b>破棄</b>(_destruct_)する。
///
/// 破棄対象への他スレッドでのアクセスが存在する場合、それらがすべて終わるのを待ってから破棄する。
///
/// この関数の呼び出し後に破棄し終えた対象にアクセスすると、プロセスを異常終了する。
///
/// @param [in] open_jtalk 破棄対象
///
/// \example{
/// ```c
/// voicevox_open_jtalk_rc_delete(open_jtalk);
/// ```
/// }
///
/// \no-orig-impl{voicevox_open_jtalk_rc_delete}
#[no_mangle]
pub extern "C" fn voicevox_open_jtalk_rc_delete(open_jtalk: *mut OpenJtalkRc) {
    init_logger_once();
    open_jtalk.drop_body();
}

/// ハードウェアアクセラレーションモードを設定する設定値。
///
/// \orig-impl{VoicevoxAccelerationMode}
#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(
    non_camel_case_types,
    reason = "実際に公開するC APIとの差異をできるだけ少なくするため"
)]
pub enum VoicevoxAccelerationMode {
    /// 実行環境に合った適切なハードウェアアクセラレーションモードを選択する
    VOICEVOX_ACCELERATION_MODE_AUTO = 0,
    /// ハードウェアアクセラレーションモードを"CPU"に設定する
    VOICEVOX_ACCELERATION_MODE_CPU = 1,
    /// ハードウェアアクセラレーションモードを"GPU"に設定する
    VOICEVOX_ACCELERATION_MODE_GPU = 2,
}

/// ::voicevox_synthesizer_new のオプション。
///
/// \no-orig-impl{VoicevoxInitializeOptions}
#[repr(C)]
pub struct VoicevoxInitializeOptions {
    /// ハードウェアアクセラレーションモード
    acceleration_mode: VoicevoxAccelerationMode,
    /// CPU利用数を指定
    /// 0を指定すると環境に合わせたCPUが利用される
    cpu_num_threads: u16,
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// デフォルトの初期化オプションを生成する
/// @return デフォルト値が設定された初期化オプション
///
/// \no-orig-impl{voicevox_make_default_initialize_options}
#[no_mangle]
pub extern "C" fn voicevox_make_default_initialize_options() -> VoicevoxInitializeOptions {
    init_logger_once();
    VoicevoxInitializeOptions::default()
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// voicevoxのバージョンを取得する。
/// @return SemVerでフォーマットされたバージョン。
///
/// \orig-impl{voicevox_get_version}
#[no_mangle]
pub extern "C" fn voicevox_get_version() -> *const c_char {
    init_logger_once();
    return C_STRING_DROP_CHECKER.blacklist(VERSION).as_ptr();

    const VERSION: &CStr = if let Ok(version) =
        CStr::from_bytes_with_nul(concat!(env!("CARGO_PKG_VERSION"), '\0').as_bytes())
    {
        version
    } else {
        panic!("`$CARGO_PKG_VERSION` should be a SemVer, so it should not contain `\\0`");
    };
}

/// 音声モデルファイル。
///
/// VVMファイルと対応する。
/// <b>構築</b>(_construction_)は ::voicevox_voice_model_file_open で行い、<b>破棄</b>(_destruction_)は ::voicevox_voice_model_file_delete で行う。
///
/// \orig-impl{VoicevoxVoiceModelFile}
#[derive(Debug, Educe)]
#[educe(Default(expression = "Self { _padding: MaybeUninit::uninit() }"))]
pub struct VoicevoxVoiceModelFile {
    _padding: MaybeUninit<[u8; 1]>,
}

/// 音声モデルID。
///
/// \orig-impl{VoicevoxVoiceModelId}
pub type VoicevoxVoiceModelId<'a> = &'a [u8; 16];

/// スタイルID。
///
/// VOICEVOXにおける、ある<i>キャラクター</i>のある<i>スタイル</i>を指す。
///
/// \orig-impl{VoicevoxStyleId}
pub type VoicevoxStyleId = u32;

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// VVMファイルを開く。
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
///
/// \orig-impl{voicevox_voice_model_file_open}
#[no_mangle]
pub unsafe extern "C" fn voicevox_voice_model_file_open(
    path: *const c_char,
    out_model: NonNull<NonNull<VoicevoxVoiceModelFile>>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let path = ensure_utf8(CStr::from_ptr(path))?;
        let model = VoicevoxVoiceModelFile::open(path)?;
        out_model.write_unaligned(model);
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ::VoicevoxVoiceModelFile からIDを取得する。
///
/// @param [in] model 音声モデル
/// @param [out] output_voice_model_id 音声モデルID
///
/// \safety{
/// - `output_voice_model_id`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_voice_model_file_id}
#[no_mangle]
pub unsafe extern "C" fn voicevox_voice_model_file_id(
    model: *const VoicevoxVoiceModelFile,
    output_voice_model_id: NonNull<[u8; 16]>,
) {
    init_logger_once();
    let id = model.body().id().0.into_bytes();
    unsafe { output_voice_model_id.write_unaligned(id) };
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ::VoicevoxVoiceModelFile からメタ情報を取得する。
///
/// JSONの解放は ::voicevox_json_free で行う。
///
/// @param [in] model 音声モデル
///
/// @returns メタ情報のJSON文字列
///
/// \orig-impl{voicevox_voice_model_file_create_metas_json}
#[no_mangle]
pub extern "C" fn voicevox_voice_model_file_create_metas_json(
    model: *const VoicevoxVoiceModelFile,
) -> *mut c_char {
    init_logger_once();
    C_STRING_DROP_CHECKER.whitelist(model.metas()).into_raw()
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ::VoicevoxVoiceModelFile を、所有しているファイルディスクリプタを閉じた上で<b>破棄</b>(_destruct_)する。ファイルの削除(_delete_)<b>ではない</b>。
///
/// 破棄対象への他スレッドでのアクセスが存在する場合、それらがすべて終わるのを待ってから破棄する。
///
/// この関数の呼び出し後に破棄し終えた対象にアクセスすると、プロセスを異常終了する。
///
/// @param [in] model 破棄対象
///
/// \no-orig-impl{voicevox_voice_model_file_delete}
#[no_mangle]
pub extern "C" fn voicevox_voice_model_file_delete(model: *mut VoicevoxVoiceModelFile) {
    init_logger_once();
    model.drop_body();
}

/// 音声シンセサイザ。
///
/// <b>構築</b>(_construction_)は ::voicevox_synthesizer_new で行い、<b>破棄</b>(_destruction_)は ::voicevox_synthesizer_delete で行う。
///
/// \orig-impl{VoicevoxSynthesizer}
#[derive(Debug, Educe)]
#[educe(Default(expression = "Self { _padding: MaybeUninit::uninit() }"))]
pub struct VoicevoxSynthesizer {
    _padding: MaybeUninit<[u8; 1]>,
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ::VoicevoxSynthesizer を<b>構築</b>(_construct_)する。
///
/// @param [in] onnxruntime
/// @param [in] open_jtalk Open JTalkのオブジェクト
/// @param [in] options オプション
/// @param [out] out_synthesizer 構築先
///
/// @returns 結果コード
///
/// \safety{
/// - `onnxruntime`は ::voicevox_onnxruntime_load_once または ::voicevox_onnxruntime_init_once で得たものでなければならない。
/// - `out_synthesizer`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_new}
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_new(
    onnxruntime: &'static VoicevoxOnnxruntime,
    open_jtalk: *const OpenJtalkRc,
    options: VoicevoxInitializeOptions,
    out_synthesizer: NonNull<NonNull<VoicevoxSynthesizer>>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let synthesizer = VoicevoxSynthesizer::new(onnxruntime, open_jtalk, options)?;
        out_synthesizer.write_unaligned(synthesizer);
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ::VoicevoxSynthesizer を<b>破棄</b>(_destruct_)する。
///
/// 破棄対象への他スレッドでのアクセスが存在する場合、それらがすべて終わるのを待ってから破棄する。
///
/// この関数の呼び出し後に破棄し終えた対象にアクセスすると、プロセスを異常終了する。
///
/// @param [in] synthesizer 破棄対象
///
/// \no-orig-impl{voicevox_synthesizer_delete}
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_delete(synthesizer: *mut VoicevoxSynthesizer) {
    init_logger_once();
    synthesizer.drop_body();
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// 音声モデルを読み込む。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] model 音声モデル
///
/// @returns 結果コード
///
/// \orig-impl{voicevox_synthesizer_load_voice_model}
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_load_voice_model(
    synthesizer: *const VoicevoxSynthesizer,
    model: *const VoicevoxVoiceModelFile,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error(synthesizer.load_voice_model(&model.body()))
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// 音声モデルの読み込みを解除する。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] model_id 音声モデルID
///
/// @returns 結果コード
///
/// \safety{
/// - `model_id`は<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_unload_voice_model}
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_unload_voice_model(
    synthesizer: *const VoicevoxSynthesizer,
    model_id: VoicevoxVoiceModelId<'_>,
) -> VoicevoxResultCode {
    init_logger_once();
    let model_id = model_id.to_model_id();
    into_result_code_with_error(synthesizer.unload_voice_model(model_id).map_err(Into::into))
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ::VoicevoxOnnxruntime のインスタンスを得る。
///
/// @param [in] synthesizer 音声シンセサイザ
///
/// @returns ::VoicevoxOnnxruntime のインスタンス
///
/// \orig-impl{voicevox_synthesizer_get_onnxruntime}
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_get_onnxruntime(
    synthesizer: *const VoicevoxSynthesizer,
) -> &'static VoicevoxOnnxruntime {
    synthesizer.onnxruntime()
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ハードウェアアクセラレーションがGPUモードか判定する。
///
/// @param [in] synthesizer 音声シンセサイザ
///
/// @returns GPUモードかどうか
///
/// \orig-impl{voicevox_synthesizer_is_gpu_mode}
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_is_gpu_mode(
    synthesizer: *const VoicevoxSynthesizer,
) -> bool {
    init_logger_once();
    synthesizer.body().is_gpu_mode()
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// 指定したIDの音声モデルが読み込まれているか判定する。
///
/// @param [in] synthesizer 音声シンセサイザ
/// @param [in] model_id 音声モデルID
///
/// @returns モデルが読み込まれているかどうか
///
/// \safety{
/// - `model_id`は<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_is_loaded_voice_model}
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_is_loaded_voice_model(
    synthesizer: *const VoicevoxSynthesizer,
    model_id: VoicevoxVoiceModelId<'_>,
) -> bool {
    init_logger_once();
    let model_id = model_id.to_model_id();
    synthesizer.body().is_loaded_voice_model(model_id)
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// 今読み込んでいる音声モデルのメタ情報を、JSONで取得する。
///
/// JSONの解放は ::voicevox_json_free で行う。
///
/// @param [in] synthesizer 音声シンセサイザ
///
/// @return メタ情報のJSON文字列
///
/// \orig-impl{voicevox_synthesizer_create_metas_json}
#[no_mangle]
pub extern "C" fn voicevox_synthesizer_create_metas_json(
    synthesizer: *const VoicevoxSynthesizer,
) -> *mut c_char {
    init_logger_once();
    let metas = synthesizer.metas();
    C_STRING_DROP_CHECKER.whitelist(metas).into_raw()
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ONNX Runtimeとして利用可能なデバイスの情報を、JSONで取得する。
///
/// JSONの解放は ::voicevox_json_free で行う。
///
/// あくまでONNX Runtimeが対応しているデバイスの情報であることに注意。GPUが使える環境ではなかったとしても`cuda`や`dml`は`true`を示しうる。
///
/// @param [in] onnxruntime
/// @param [out] output_supported_devices_json サポートデバイス情報のJSON文字列
///
/// @returns 結果コード
///
/// \example{
/// ```c
/// char *supported_devices;
/// VoicevoxResultCode result = voicevox_onnxruntime_create_supported_devices_json(onnxruntime, &supported_devices);
/// ```
/// }
///
/// \safety{
/// - `onnxruntime`は ::voicevox_onnxruntime_load_once または ::voicevox_onnxruntime_init_once で得たものでなければならない。
/// - `output_supported_devices_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_onnxruntime_create_supported_devices_json}
#[no_mangle]
pub unsafe extern "C" fn voicevox_onnxruntime_create_supported_devices_json(
    onnxruntime: &'static VoicevoxOnnxruntime,
    output_supported_devices_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let supported_devices = CString::new(onnxruntime.0.supported_devices()?.to_json()).unwrap();
        output_supported_devices_json.write_unaligned(
            C_STRING_DROP_CHECKER
                .whitelist(supported_devices)
                .into_raw(),
        );
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
/// - `kana`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_create_audio_query_from_kana}
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_create_audio_query_from_kana(
    synthesizer: *const VoicevoxSynthesizer,
    kana: *const c_char,
    style_id: VoicevoxStyleId,
    output_audio_query_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let kana = CStr::from_ptr(kana);
        let kana = ensure_utf8(kana)?;

        let audio_query = synthesizer
            .body()
            .create_audio_query_from_kana(kana, StyleId::new(style_id))?;
        let audio_query = CString::new(audio_query_model_to_json(&audio_query))
            .expect("should not contain '\\0'");
        output_audio_query_json
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(audio_query).into_raw());
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
/// - `text`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_create_audio_query}
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_create_audio_query(
    synthesizer: *const VoicevoxSynthesizer,
    text: *const c_char,
    style_id: VoicevoxStyleId,
    output_audio_query_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let text = CStr::from_ptr(text);
        let text = ensure_utf8(text)?;

        let audio_query = synthesizer
            .body()
            .create_audio_query(text, StyleId::new(style_id))?;
        let audio_query = CString::new(audio_query_model_to_json(&audio_query))
            .expect("should not contain '\\0'");
        output_audio_query_json
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(audio_query).into_raw());
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
/// - `kana`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_create_accent_phrases_from_kana}
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_create_accent_phrases_from_kana(
    synthesizer: *const VoicevoxSynthesizer,
    kana: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let kana = ensure_utf8(CStr::from_ptr(kana))?;
        let accent_phrases = synthesizer
            .body()
            .create_accent_phrases_from_kana(kana, StyleId::new(style_id))?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
/// - `text`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_create_accent_phrases}
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_create_accent_phrases(
    synthesizer: *const VoicevoxSynthesizer,
    text: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let text = ensure_utf8(CStr::from_ptr(text))?;
        let accent_phrases = synthesizer
            .body()
            .create_accent_phrases(text, StyleId::new(style_id))?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
/// - `accent_phrases_json`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_replace_mora_data}
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_replace_mora_data(
    synthesizer: *const VoicevoxSynthesizer,
    accent_phrases_json: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let accent_phrases: Vec<AccentPhrase> =
            serde_json::from_str(ensure_utf8(CStr::from_ptr(accent_phrases_json))?)
                .map_err(CApiError::InvalidAccentPhrase)?;
        let accent_phrases = synthesizer
            .body()
            .replace_mora_data(&accent_phrases, StyleId::new(style_id))?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
/// - `accent_phrases_json`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_replace_phoneme_length}
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_replace_phoneme_length(
    synthesizer: *const VoicevoxSynthesizer,
    accent_phrases_json: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let accent_phrases: Vec<AccentPhrase> =
            serde_json::from_str(ensure_utf8(CStr::from_ptr(accent_phrases_json))?)
                .map_err(CApiError::InvalidAccentPhrase)?;
        let accent_phrases = synthesizer
            .body()
            .replace_phoneme_length(&accent_phrases, StyleId::new(style_id))?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
/// - `accent_phrases_json`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_audio_query_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_replace_mora_pitch}
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_replace_mora_pitch(
    synthesizer: *const VoicevoxSynthesizer,
    accent_phrases_json: *const c_char,
    style_id: VoicevoxStyleId,
    output_accent_phrases_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let accent_phrases: Vec<AccentPhrase> =
            serde_json::from_str(ensure_utf8(CStr::from_ptr(accent_phrases_json))?)
                .map_err(CApiError::InvalidAccentPhrase)?;
        let accent_phrases = synthesizer
            .body()
            .replace_mora_pitch(&accent_phrases, StyleId::new(style_id))?;
        let accent_phrases = CString::new(accent_phrases_to_json(&accent_phrases))
            .expect("should not contain '\\0'");
        output_accent_phrases_json
            .write_unaligned(C_STRING_DROP_CHECKER.whitelist(accent_phrases).into_raw());
        Ok(())
    })())
}

/// ::voicevox_synthesizer_synthesis のオプション。
///
/// \no-orig-impl{VoicevoxSynthesisOptions}
#[repr(C)]
pub struct VoicevoxSynthesisOptions {
    /// 疑問文の調整を有効にする
    enable_interrogative_upspeak: bool,
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// デフォルトの `voicevox_synthesizer_synthesis` のオプションを生成する
/// @return デフォルト値が設定された `voicevox_synthesizer_synthesis` のオプション
///
/// \no-orig-impl{voicevox_make_default_synthesis_options}
#[no_mangle]
pub extern "C" fn voicevox_make_default_synthesis_options() -> VoicevoxSynthesisOptions {
    init_logger_once();
    VoicevoxSynthesisOptions::default()
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
/// - `audio_query_json`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_wav_length`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// - `output_wav`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_synthesis}
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_synthesis(
    synthesizer: *const VoicevoxSynthesizer,
    audio_query_json: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxSynthesisOptions,
    output_wav_length: NonNull<usize>,
    output_wav: NonNull<NonNull<u8>>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let audio_query_json = CStr::from_ptr(audio_query_json)
            .to_str()
            .map_err(|_| CApiError::InvalidUtf8Input)?;
        let audio_query: AudioQuery =
            serde_json::from_str(audio_query_json).map_err(CApiError::InvalidAudioQuery)?;
        let VoicevoxSynthesisOptions {
            enable_interrogative_upspeak,
        } = options;
        let wav = synthesizer
            .body()
            .synthesis(&audio_query, StyleId::new(style_id))
            .enable_interrogative_upspeak(enable_interrogative_upspeak)
            .perform()?;
        U8_SLICE_OWNER.own_and_lend(wav, output_wav, output_wav_length);
        Ok(())
    })())
}

/// ::voicevox_synthesizer_tts のオプション。
///
/// \no-orig-impl{VoicevoxTtsOptions}
#[repr(C)]
pub struct VoicevoxTtsOptions {
    /// 疑問文の調整を有効にする
    enable_interrogative_upspeak: bool,
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// デフォルトのテキスト音声合成オプションを生成する
/// @return テキスト音声合成オプション
///
/// \no-orig-impl{voicevox_make_default_tts_options}
#[no_mangle]
pub extern "C" fn voicevox_make_default_tts_options() -> VoicevoxTtsOptions {
    init_logger_once();
    VoicevoxTtsOptions::default()
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
/// - `kana`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_wav_length`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// - `output_wav`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_tts_from_kana}
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_tts_from_kana(
    synthesizer: *const VoicevoxSynthesizer,
    kana: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxTtsOptions,
    output_wav_length: NonNull<usize>,
    output_wav: NonNull<NonNull<u8>>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let kana = ensure_utf8(CStr::from_ptr(kana))?;
        let VoicevoxTtsOptions {
            enable_interrogative_upspeak,
        } = options;
        let output = synthesizer
            .body()
            .tts_from_kana(kana, StyleId::new(style_id))
            .enable_interrogative_upspeak(enable_interrogative_upspeak)
            .perform()?;
        U8_SLICE_OWNER.own_and_lend(output, output_wav, output_wav_length);
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
/// - `text`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_wav_length`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// - `output_wav`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_synthesizer_tts}
#[no_mangle]
pub unsafe extern "C" fn voicevox_synthesizer_tts(
    synthesizer: *const VoicevoxSynthesizer,
    text: *const c_char,
    style_id: VoicevoxStyleId,
    options: VoicevoxTtsOptions,
    output_wav_length: NonNull<usize>,
    output_wav: NonNull<NonNull<u8>>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let text = ensure_utf8(CStr::from_ptr(text))?;
        let VoicevoxTtsOptions {
            enable_interrogative_upspeak,
        } = options;
        let output = synthesizer
            .body()
            .tts(text, StyleId::new(style_id))
            .enable_interrogative_upspeak(enable_interrogative_upspeak)
            .perform()?;
        U8_SLICE_OWNER.own_and_lend(output, output_wav, output_wav_length);
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// JSON文字列を解放する。
///
/// @param [in] json 解放するJSON文字列
///
/// \safety{
/// - `json`は以下のAPIで得られたポインタでなくてはいけない。
///     - ::voicevox_onnxruntime_create_supported_devices_json
///     - ::voicevox_voice_model_file_create_metas_json
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
///
/// \no-orig-impl{voicevox_json_free}
#[no_mangle]
pub unsafe extern "C" fn voicevox_json_free(json: *mut c_char) {
    init_logger_once();
    drop(CString::from_raw(C_STRING_DROP_CHECKER.check(json)));
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
///
/// \no-orig-impl{voicevox_wav_free}
#[no_mangle]
pub extern "C" fn voicevox_wav_free(wav: *mut u8) {
    init_logger_once();
    U8_SLICE_OWNER.drop_for(wav);
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
///
/// \no-orig-impl{voicevox_error_result_to_message}
#[no_mangle]
pub extern "C" fn voicevox_error_result_to_message(
    result_code: VoicevoxResultCode,
) -> *const c_char {
    init_logger_once();
    let message = result_code::error_result_to_message(result_code);
    C_STRING_DROP_CHECKER.blacklist(message).as_ptr()
}

/// ユーザー辞書。
///
/// \orig-impl{VoicevoxUserDict}
#[derive(Debug, Educe)]
#[educe(Default(expression = "Self { _padding: MaybeUninit::uninit() }"))]
pub struct VoicevoxUserDict {
    _padding: MaybeUninit<[u8; 1]>,
}

/// ユーザー辞書の単語。
///
/// \orig-impl{VoicevoxUserDictWord}
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
///
/// \orig-impl{VoicevoxUserDictWordType}
#[repr(i32)]
#[allow(
    non_camel_case_types,
    reason = "実際に公開するC APIとの差異をできるだけ少なくするため"
)]
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

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ::VoicevoxUserDictWord を最低限のパラメータで作成する。
///
/// @param [in] surface 表記
/// @param [in] pronunciation 読み
/// @param [in] accent_type アクセント型
/// @returns ::VoicevoxUserDictWord
///
/// \orig-impl{voicevox_user_dict_word_make}
#[no_mangle]
pub extern "C" fn voicevox_user_dict_word_make(
    surface: *const c_char,
    pronunciation: *const c_char,
    accent_type: usize,
) -> VoicevoxUserDictWord {
    init_logger_once();
    VoicevoxUserDictWord {
        surface,
        pronunciation,
        accent_type,
        word_type: DEFAULT_WORD_TYPE.into(),
        priority: DEFAULT_PRIORITY,
    }
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ユーザー辞書をb>構築</b>(_construct_)する。
///
/// @returns ::VoicevoxUserDict
///
/// \orig-impl{voicevox_user_dict_new}
#[no_mangle]
pub extern "C" fn voicevox_user_dict_new() -> NonNull<VoicevoxUserDict> {
    init_logger_once();
    VoicevoxUserDict::new(Default::default())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ユーザー辞書にファイルを読み込ませる。
///
/// @param [in] user_dict ユーザー辞書
/// @param [in] dict_path 読み込む辞書ファイルのパス
/// @returns 結果コード
///
/// \safety{
/// - `dict_path`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_user_dict_load}
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_load(
    user_dict: *const VoicevoxUserDict,
    dict_path: *const c_char,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let dict_path = ensure_utf8(unsafe { CStr::from_ptr(dict_path) })?;
        user_dict.body().load(dict_path)?;

        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
/// - `word->surface`と`word->pronunciation`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `output_word_uuid`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_user_dict_add_word}
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_add_word(
    user_dict: *const VoicevoxUserDict,
    word: *const VoicevoxUserDictWord,
    output_word_uuid: NonNull<[u8; 16]>,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let word = word.read_unaligned().try_into_word()?;
        let uuid = user_dict.body().add_word(word)?;
        output_word_uuid.write_unaligned(uuid.into_bytes());

        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ユーザー辞書の単語を更新する。
///
/// @param [in] user_dict ユーザー辞書
/// @param [in] word_uuid 更新する単語のUUID
/// @param [in] word 新しい単語のデータ
/// @returns 結果コード
///
/// \safety{
/// - `word_uuid`は<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// - `word->surface`と`word->pronunciation`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_user_dict_update_word}
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_update_word(
    user_dict: *const VoicevoxUserDict,
    word_uuid: &[u8; 16],
    word: *const VoicevoxUserDictWord,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let word_uuid = Uuid::from_slice(word_uuid).map_err(CApiError::InvalidUuid)?;
        let word = word.read_unaligned().try_into_word()?;
        user_dict.body().update_word(word_uuid, word)?;

        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ユーザー辞書から単語を削除する。
///
/// @param [in] user_dict ユーザー辞書
/// @param [in] word_uuid 削除する単語のUUID
/// @returns 結果コード
///
/// \safety{
/// - `word_uuid`は<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_user_dict_remove_word}
#[no_mangle]
pub extern "C" fn voicevox_user_dict_remove_word(
    user_dict: *const VoicevoxUserDict,
    word_uuid: &[u8; 16],
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let word_uuid = Uuid::from_slice(word_uuid).map_err(CApiError::InvalidUuid)?;
        user_dict.body().remove_word(word_uuid)?;
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
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
/// - `output_json`は<a href="#voicevox-core-safety">書き込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_user_dict_to_json}
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_to_json(
    user_dict: *const VoicevoxUserDict,
    output_json: NonNull<*mut c_char>,
) -> VoicevoxResultCode {
    init_logger_once();
    let json = user_dict.body().to_json();
    let json = CString::new(json).expect("\\0を含まない文字列であることが保証されている");
    output_json.write_unaligned(C_STRING_DROP_CHECKER.whitelist(json).into_raw());
    VoicevoxResultCode::VOICEVOX_RESULT_OK
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// 他のユーザー辞書をインポートする。
///
/// @param [in] user_dict ユーザー辞書
/// @param [in] other_dict インポートするユーザー辞書
/// @returns 結果コード
///
/// \orig-impl{voicevox_user_dict_import}
#[no_mangle]
pub extern "C" fn voicevox_user_dict_import(
    user_dict: *const VoicevoxUserDict,
    other_dict: *const VoicevoxUserDict,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        user_dict.body().import(&other_dict.body())?;
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ユーザー辞書をファイルに保存する。
///
/// @param [in] user_dict ユーザー辞書
/// @param [in] path 保存先のファイルパス
///
/// \safety{
/// - `path`はヌル終端文字列を指し、かつ<a href="#voicevox-core-safety">読み込みについて有効</a>でなければならない。
/// }
///
/// \orig-impl{voicevox_user_dict_save}
#[no_mangle]
pub unsafe extern "C" fn voicevox_user_dict_save(
    user_dict: *const VoicevoxUserDict,
    path: *const c_char,
) -> VoicevoxResultCode {
    init_logger_once();
    into_result_code_with_error((|| {
        let path = ensure_utf8(CStr::from_ptr(path))?;
        user_dict.body().save(path)?;
        Ok(())
    })())
}

// TODO: cbindgenが`#[unsafe(no_mangle)]`に対応したら`#[no_mangle]`を置き換える
// SAFETY: voicevox_core_c_apiを構成するライブラリの中に、これと同名のシンボルは存在しない
/// ユーザー辞書を<b>破棄</b>(_destruct_)する。
///
/// 破棄対象への他スレッドでのアクセスが存在する場合、それらがすべて終わるのを待ってから破棄する。
///
/// この関数の呼び出し後に破棄し終えた対象にアクセスすると、プロセスを異常終了する。
///
/// @param [in] user_dict 破棄対象
///
/// \no-orig-impl{voicevox_user_dict_delete}
#[no_mangle]
pub extern "C" fn voicevox_user_dict_delete(user_dict: *mut VoicevoxUserDict) {
    init_logger_once();
    user_dict.drop_body();
}
