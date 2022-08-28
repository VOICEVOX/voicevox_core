/// voicevox_coreの処理結果を示す結果コード
#[repr(i32)]
#[derive(Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum VoicevoxResultCode {
    // C でのenum定義に合わせて大文字で定義している
    // 出力フォーマットを変更すればRustでよく使われているUpperCamelにできるが、実際に出力されるコードとの差異をできるだけ少なくするため
    /// 成功
    VOICEVOX_RESULT_SUCCEED = 0,
    /// open_jtalk辞書ファイルが読み込まれていない
    VOICEVOX_RESULT_NOT_LOADED_OPENJTALK_DICT = 1,
    /// modelの読み込みに失敗した
    VOICEVOX_RESULT_FAILED_LOAD_MODEL = 2,
    /// サポートされているデバイス情報取得に失敗した
    VOICEVOX_RESULT_FAILED_GET_SUPPORTED_DEVICES = 3,
    /// GPUモードがサポートされていない
    VOICEVOX_RESULT_CANT_GPU_SUPPORT = 4,
    /// メタ情報読み込みに失敗した
    VOICEVOX_RESULT_FAILED_LOAD_METAS = 5,
    /// ステータスが初期化されていない
    VOICEVOX_RESULT_UNINITIALIZED_STATUS = 6,
    /// 無効なspeaker_idが指定された
    VOICEVOX_RESULT_INVALID_SPEAKER_ID = 7,
    /// 無効なmodel_indexが指定された
    VOICEVOX_RESULT_INVALID_MODEL_INDEX = 8,
    /// 推論に失敗した
    VOICEVOX_RESULT_INFERENCE_FAILED = 9,
    /// コンテキストラベル出力に失敗した
    VOICEVOX_RESULT_FAILED_EXTRACT_FULL_CONTEXT_LABEL = 10,
    /// 無効なutf8文字列が入力された
    VOICEVOX_RESULT_INVALID_UTF8_INPUT = 11,
    /// aquestalk形式のテキストの解析に失敗した
    VOICEVOX_RESULT_FAILED_PARSE_KANA = 12,
    /// 無効なAudioQuery
    VOICEVOX_RESULT_INVALID_AUDIO_QUERY = 13,
}
