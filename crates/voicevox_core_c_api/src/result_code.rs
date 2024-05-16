use std::ffi::CStr;

use cstr::cstr;

/// 処理結果を示す結果コード。
#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum VoicevoxResultCode {
    // C でのenum定義に合わせて大文字で定義している
    // 出力フォーマットを変更すればRustでよく使われているUpperCamelにできるが、実際に出力されるコードとの差異をできるだけ少なくするため
    /// 成功
    VOICEVOX_RESULT_OK = 0,
    /// open_jtalk辞書ファイルが読み込まれていない
    VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR = 1,
    /// サポートされているデバイス情報取得に失敗した
    VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR = 3,
    /// GPUモードがサポートされていない
    VOICEVOX_RESULT_GPU_SUPPORT_ERROR = 4,
    /// スタイルIDに対するスタイルが見つからなかった
    VOICEVOX_RESULT_STYLE_NOT_FOUND_ERROR = 6,
    /// 音声モデルIDに対する音声モデルが見つからなかった
    VOICEVOX_RESULT_MODEL_NOT_FOUND_ERROR = 7,
    /// 推論に失敗した
    VOICEVOX_RESULT_INFERENCE_ERROR = 8,
    /// コンテキストラベル出力に失敗した
    VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR = 11,
    /// 無効なutf8文字列が入力された
    VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR = 12,
    /// AquesTalk風記法のテキストの解析に失敗した
    VOICEVOX_RESULT_PARSE_KANA_ERROR = 13,
    /// 無効なAudioQuery
    VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR = 14,
    /// 無効なAccentPhrase
    VOICEVOX_RESULT_INVALID_ACCENT_PHRASE_ERROR = 15,
    /// ZIPファイルを開くことに失敗した
    VOICEVOX_RESULT_OPEN_ZIP_FILE_ERROR = 16,
    /// ZIP内のファイルが読めなかった
    VOICEVOX_RESULT_READ_ZIP_ENTRY_ERROR = 17,
    /// モデルの形式が不正
    VOICEVOX_RESULT_INVALID_MODEL_HEADER_ERROR = 28,
    /// すでに読み込まれている音声モデルを読み込もうとした
    VOICEVOX_RESULT_MODEL_ALREADY_LOADED_ERROR = 18,
    /// すでに読み込まれているスタイルを読み込もうとした
    VOICEVOX_RESULT_STYLE_ALREADY_LOADED_ERROR = 26,
    /// 無効なモデルデータ
    VOICEVOX_RESULT_INVALID_MODEL_DATA_ERROR = 27,
    /// ユーザー辞書を読み込めなかった
    VOICEVOX_RESULT_LOAD_USER_DICT_ERROR = 20,
    /// ユーザー辞書を書き込めなかった
    VOICEVOX_RESULT_SAVE_USER_DICT_ERROR = 21,
    /// ユーザー辞書に単語が見つからなかった
    VOICEVOX_RESULT_USER_DICT_WORD_NOT_FOUND_ERROR = 22,
    /// OpenJTalkのユーザー辞書の設定に失敗した
    VOICEVOX_RESULT_USE_USER_DICT_ERROR = 23,
    /// ユーザー辞書の単語のバリデーションに失敗した
    VOICEVOX_RESULT_INVALID_USER_DICT_WORD_ERROR = 24,
    /// UUIDの変換に失敗した
    VOICEVOX_RESULT_INVALID_UUID_ERROR = 25,
}

pub(crate) const fn error_result_to_message(result_code: VoicevoxResultCode) -> &'static CStr {
    use VoicevoxResultCode::*;
    match result_code {
        VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR => {
            cstr!("OpenJTalkの辞書が読み込まれていません")
        }
        VOICEVOX_RESULT_GPU_SUPPORT_ERROR => cstr!("GPU機能をサポートすることができません"),
        VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR => {
            cstr!("サポートされているデバイス情報取得中にエラーが発生しました")
        }
        VOICEVOX_RESULT_OK => cstr!("エラーが発生しませんでした"),
        VOICEVOX_RESULT_STYLE_NOT_FOUND_ERROR => cstr!(
            "指定されたIDに対するスタイルが見つかりませんでした。音声モデルが読み込まれていないか\
             、読み込みが解除されています"
        ),
        VOICEVOX_RESULT_MODEL_NOT_FOUND_ERROR => cstr!(
            "指定されたIDに対する音声モデルが見つかりませんでした。読み込まれていないか、読み込み\
             が既に解除されています"
        ),
        VOICEVOX_RESULT_INFERENCE_ERROR => cstr!("推論に失敗しました"),
        VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR => {
            cstr!("入力テキストからのフルコンテキストラベル抽出に失敗しました")
        }
        VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR => cstr!("入力テキストが無効なUTF-8データでした"),
        VOICEVOX_RESULT_PARSE_KANA_ERROR => {
            cstr!("入力テキストをAquesTalk風記法としてパースすることに失敗しました")
        }
        VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR => cstr!("無効なaudio_queryです"),
        VOICEVOX_RESULT_INVALID_ACCENT_PHRASE_ERROR => cstr!("無効なaccent_phraseです"),
        VOICEVOX_RESULT_OPEN_ZIP_FILE_ERROR => cstr!("ZIPファイルのオープンに失敗しました"),
        VOICEVOX_RESULT_READ_ZIP_ENTRY_ERROR => {
            cstr!("ZIP内のファイルを読むことができませんでした")
        }
        VOICEVOX_RESULT_INVALID_MODEL_HEADER_ERROR => cstr!("モデルの形式が不正です"),
        VOICEVOX_RESULT_MODEL_ALREADY_LOADED_ERROR => cstr!("同じIDのモデルを読むことはできません"),
        VOICEVOX_RESULT_STYLE_ALREADY_LOADED_ERROR => {
            cstr!("同じIDのスタイルを読むことはできません")
        }
        VOICEVOX_RESULT_INVALID_MODEL_DATA_ERROR => {
            cstr!("モデルデータを読むことができませんでした")
        }
        VOICEVOX_RESULT_LOAD_USER_DICT_ERROR => cstr!("ユーザー辞書を読み込めませんでした"),
        VOICEVOX_RESULT_SAVE_USER_DICT_ERROR => cstr!("ユーザー辞書を書き込めませんでした"),
        VOICEVOX_RESULT_USER_DICT_WORD_NOT_FOUND_ERROR => {
            cstr!("ユーザー辞書に単語が見つかりませんでした")
        }
        VOICEVOX_RESULT_USE_USER_DICT_ERROR => cstr!("OpenJTalkのユーザー辞書の設定に失敗しました"),
        VOICEVOX_RESULT_INVALID_USER_DICT_WORD_ERROR => {
            cstr!("ユーザー辞書の単語のバリデーションに失敗しました")
        }
        VOICEVOX_RESULT_INVALID_UUID_ERROR => cstr!("UUIDの変換に失敗しました"),
    }
}
