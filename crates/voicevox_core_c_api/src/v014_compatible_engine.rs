use super::*;

/// ::voicevox_initialize のオプション。
#[deprecated(note = "VoicevoxInitializeSynthesizerOptions を使ってください。")]
#[repr(C)]
pub struct VoicevoxInitializeOptions {
    /// ハードウェアアクセラレーションモード
    acceleration_mode: VoicevoxAccelerationMode,
    /// CPU利用数を指定
    /// 0を指定すると環境に合わせたCPUが利用される
    cpu_num_threads: u16,
    /// 全てのモデルを読み込む
    load_all_models: bool,
    /// open_jtalkの辞書ディレクトリ
    open_jtalk_dict_dir: *const c_char,
}

#[no_mangle]
#[deprecated(note = "voicevox_synthesizer_new_with_initialize を使ってください。")]
/// 初期化する。
///
/// @param [in] options オプション
/// @returns 結果コード
pub extern "C" fn voicevox_initialize(options: VoicevoxInitializeOptions) -> VoicevoxResultCode {
    todo!()
}
