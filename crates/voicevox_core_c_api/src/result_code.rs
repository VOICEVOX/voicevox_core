use std::ffi::CStr;

/// 処理結果を示す結果コード。
///
/// \orig-impl{VoicevoxResultCode,C APIにしか無いものがあることに注意。}
#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(
    non_camel_case_types,
    reason = "実際に公開するC APIとの差異をできるだけ少なくするため"
)]
pub enum VoicevoxResultCode {
    /// 成功
    VOICEVOX_RESULT_OK = 0,
    /// open_jtalk辞書ファイルが読み込まれていない
    VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR = 1,
    /// サポートされているデバイス情報取得に失敗した
    VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR = 3,
    /// GPUモードがサポートされていない
    VOICEVOX_RESULT_GPU_SUPPORT_ERROR = 4,
    /// 推論ライブラリのロードまたは初期化ができなかった
    VOICEVOX_RESULT_INIT_INFERENCE_RUNTIME_ERROR = 29,
    /// スタイルIDに対するスタイルが見つからなかった
    VOICEVOX_RESULT_STYLE_NOT_FOUND_ERROR = 6,
    /// 音声モデルIDに対する音声モデルが見つからなかった
    VOICEVOX_RESULT_MODEL_NOT_FOUND_ERROR = 7,
    /// 推論に失敗した、もしくは推論結果が異常
    VOICEVOX_RESULT_RUN_MODEL_ERROR = 8,
    /// 入力テキストの解析に失敗した
    VOICEVOX_RESULT_ANALYZE_TEXT_ERROR = 11,
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
    /// 無効なMora
    VOICEVOX_RESULT_INVALID_MORA_ERROR = 30,
    /// 楽譜とFrameAudioQueryの組み合わせが不正
    VOICEVOX_RESULT_INCOMPATIBLE_QUERIES_ERROR = 35,
}

pub(crate) const fn error_result_to_message(result_code: VoicevoxResultCode) -> &'static CStr {
    use VoicevoxResultCode::*;
    match result_code {
        VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR => c"OpenJTalkの辞書が読み込まれていません",
        VOICEVOX_RESULT_GPU_SUPPORT_ERROR => c"GPU機能をサポートすることができません",
        VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR => {
            c"サポートされているデバイス情報取得中にエラーが発生しました"
        }
        VOICEVOX_RESULT_INIT_INFERENCE_RUNTIME_ERROR => {
            c"推論ライブラリのロードまたは初期化ができませんでした"
        }
        VOICEVOX_RESULT_OK => c"エラーが発生しませんでした",
        VOICEVOX_RESULT_STYLE_NOT_FOUND_ERROR => {
            c"指定されたIDに対するスタイルが見つかりませんでした。音声モデルが読み込まれていない\
              か、読み込みが解除されています"
        }
        VOICEVOX_RESULT_MODEL_NOT_FOUND_ERROR => {
            c"指定されたIDに対する音声モデルが見つかりませんでした。読み込まれていないか、読み込み\
              が既に解除されています"
        }
        VOICEVOX_RESULT_RUN_MODEL_ERROR => c"正常に推論することができませんでした",
        VOICEVOX_RESULT_ANALYZE_TEXT_ERROR => c"入力テキストの解析に失敗しました",
        VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR => c"入力テキストが無効なUTF-8データでした",
        VOICEVOX_RESULT_PARSE_KANA_ERROR => {
            c"入力テキストをAquesTalk風記法としてパースすることに失敗しました"
        }
        VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR => c"無効なaudio_queryです",
        VOICEVOX_RESULT_INVALID_ACCENT_PHRASE_ERROR => c"無効なaccent_phraseです",
        VOICEVOX_RESULT_OPEN_ZIP_FILE_ERROR => c"ZIPファイルのオープンに失敗しました",
        VOICEVOX_RESULT_READ_ZIP_ENTRY_ERROR => c"ZIP内のファイルを読むことができませんでした",
        VOICEVOX_RESULT_INVALID_MODEL_HEADER_ERROR => c"モデルの形式が不正です",
        VOICEVOX_RESULT_MODEL_ALREADY_LOADED_ERROR => c"同じIDのモデルを読むことはできません",
        VOICEVOX_RESULT_STYLE_ALREADY_LOADED_ERROR => c"同じIDのスタイルを読むことはできません",
        VOICEVOX_RESULT_INVALID_MODEL_DATA_ERROR => c"モデルデータを読むことができませんでした",
        VOICEVOX_RESULT_LOAD_USER_DICT_ERROR => c"ユーザー辞書を読み込めませんでした",
        VOICEVOX_RESULT_SAVE_USER_DICT_ERROR => c"ユーザー辞書を書き込めませんでした",
        VOICEVOX_RESULT_USER_DICT_WORD_NOT_FOUND_ERROR => {
            c"ユーザー辞書に単語が見つかりませんでした"
        }
        VOICEVOX_RESULT_USE_USER_DICT_ERROR => c"OpenJTalkのユーザー辞書の設定に失敗しました",
        VOICEVOX_RESULT_INVALID_USER_DICT_WORD_ERROR => {
            c"ユーザー辞書の単語のバリデーションに失敗しました"
        }
        VOICEVOX_RESULT_INVALID_UUID_ERROR => c"UUIDの変換に失敗しました",
        VOICEVOX_RESULT_INVALID_MORA_ERROR => c"無効なモーラです",
        VOICEVOX_RESULT_INCOMPATIBLE_QUERIES_ERROR => {
            c"不正な楽譜とFrameAudioQueryの組み合わせです。\
              これらは同じ音素列から成り立っている必要があります"
        }
    }
}
