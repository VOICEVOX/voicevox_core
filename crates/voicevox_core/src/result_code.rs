use strum::EnumIter;

/// 処理結果を示す結果コード。
#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter)]
#[allow(non_camel_case_types)]
pub enum VoicevoxResultCode {
    // C でのenum定義に合わせて大文字で定義している
    // 出力フォーマットを変更すればRustでよく使われているUpperCamelにできるが、実際に出力されるコードとの差異をできるだけ少なくするため
    /// 成功
    VOICEVOX_RESULT_OK = 0,
    /// open_jtalk辞書ファイルが読み込まれていない
    VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR = 1,
    /// modelの読み込みに失敗した
    VOICEVOX_RESULT_LOAD_MODEL_ERROR = 2,
    /// サポートされているデバイス情報取得に失敗した
    VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR = 3,
    /// GPUモードがサポートされていない
    VOICEVOX_RESULT_GPU_SUPPORT_ERROR = 4,
    /// メタ情報読み込みに失敗した
    VOICEVOX_RESULT_LOAD_METAS_ERROR = 5,
    /// 無効なstyle_idが指定された
    VOICEVOX_RESULT_INVALID_STYLE_ID_ERROR = 6,
    /// 無効なmodel_indexが指定された
    VOICEVOX_RESULT_INVALID_MODEL_INDEX_ERROR = 7,
    /// 推論に失敗した
    VOICEVOX_RESULT_INFERENCE_ERROR = 8,
    /// コンテキストラベル出力に失敗した
    VOICEVOX_RESULT_EXTRACT_FULL_CONTEXT_LABEL_ERROR = 11,
    /// 無効なutf8文字列が入力された
    VOICEVOX_RESULT_INVALID_UTF8_INPUT_ERROR = 12,
    /// aquestalk形式のテキストの解析に失敗した
    VOICEVOX_RESULT_PARSE_KANA_ERROR = 13,
    /// 無効なAudioQuery
    VOICEVOX_RESULT_INVALID_AUDIO_QUERY_ERROR = 14,
    /// 無効なAccentPhrase
    VOICEVOX_RESULT_INVALID_ACCENT_PHRASE_ERROR = 15,
    /// ファイルオープンエラー
    VOICEVOX_OPEN_FILE_ERROR = 16,
    /// Modelを読み込めなかった
    VOICEVOX_VVM_MODEL_READ_ERROR = 17,
    /// すでに読み込まれているModelを読み込もうとした
    VOICEVOX_ALREADY_LOADED_MODEL_ERROR = 18,
    /// Modelが読み込まれていない
    VOICEVOX_UNLOADED_MODEL_ERROR = 19,
}

pub const fn error_result_to_message(result_code: VoicevoxResultCode) -> &'static str {
    // C APIのため、messageには必ず末尾にNULL文字を追加する
    use VoicevoxResultCode::*;
    match result_code {
        VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT_ERROR => {
            "OpenJTalkの辞書が読み込まれていません\0"
        }
        VOICEVOX_RESULT_LOAD_MODEL_ERROR => "modelデータ読み込みに失敗しました\0",
        VOICEVOX_RESULT_LOAD_METAS_ERROR => "メタデータ読み込みに失敗しました\0",

        VOICEVOX_RESULT_GPU_SUPPORT_ERROR => "GPU機能をサポートすることができません\0",
        VOICEVOX_RESULT_GET_SUPPORTED_DEVICES_ERROR => {
            "サポートされているデバイス情報取得中にエラーが発生しました\0"
        }

        VOICEVOX_RESULT_OK => "エラーが発生しませんでした\0",
        VOICEVOX_RESULT_INVALID_STYLE_ID_ERROR => "無効なspeaker_idです\0",
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
        VOICEVOX_RESULT_INVALID_ACCENT_PHRASE_ERROR => "無効なaccent_phraseです\0",
        VOICEVOX_OPEN_FILE_ERROR => "ファイルオープンに失敗しました\0",
        VOICEVOX_VVM_MODEL_READ_ERROR => "Modelを読み込めませんでした\0",
        VOICEVOX_ALREADY_LOADED_MODEL_ERROR => {
            "すでに読み込まれているModelを読み込もうとしました\0"
        }
        VOICEVOX_UNLOADED_MODEL_ERROR => "Modelが読み込まれていません\0",
    }
}
