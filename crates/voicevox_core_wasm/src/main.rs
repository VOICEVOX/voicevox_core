//! ブラウザ(Emscripten)向けのVOICEVOX CoreのAPI。
//!
//! JavaScriptからは`ccall`で呼ぶことを想定している。
//!
//! - 文字列の引数はNUL終端のUTF-8文字列。nullやNUL終端でないポインタを渡してはならない。
//! - 各関数は成功時に`0`、失敗時に`1`を返す。
//! - 成功時は結果(JSON文字列またはWAV)が、失敗時はエラーメッセージ(UTF-8)が結果バッファに入る。
//!   結果バッファは[`voicevox_wasm_result_pointer`]と[`voicevox_wasm_result_length`]で読み出し、
//!   次の呼び出しまで保持される。
//! - ファイルはEmscripten FS上のパスで受け取る。

#![expect(
    clippy::missing_safety_doc,
    reason = "文字列の引数の要件はモジュールのドキュメントにまとめて書いている"
)]

use std::{
    ffi::{CStr, c_char},
    sync::{Mutex, OnceLock},
};

use anyhow::{Context as _, ensure};
use voicevox_core::{
    AccelerationMode, AccentPhrase, AudioQuery, OnExistingVoiceModelId, StyleId,
    blocking::{Onnxruntime, OpenJtalk, Synthesizer, VoiceModelFile},
};

static SYNTHESIZER: OnceLock<Synthesizer<OpenJtalk>> = OnceLock::new();
static RESULT: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// ONNX RuntimeとOpen JTalkを初期化する。
///
/// `open_jtalk_dict_dir`は辞書を書き込んだEmscripten FS上のディレクトリ。
/// 初期化済みの場合は何もしない。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voicevox_wasm_initialize(open_jtalk_dict_dir: *const c_char) -> i32 {
    run(|| {
        if SYNTHESIZER.get().is_none() {
            let dict_dir = unsafe { to_str(open_jtalk_dict_dir) }?;
            let synthesizer = Synthesizer::builder(Onnxruntime::init_once()?)
                .text_analyzer(OpenJtalk::new(dict_dir)?)
                .acceleration_mode(AccelerationMode::Cpu)
                .cpu_num_threads(1)
                .build()?;
            let _ = SYNTHESIZER.set(synthesizer);
        }
        Ok(vec![])
    })
}

/// VVMを読み込み、そのVVMのメタ情報をJSONで返す。
///
/// 読み込み後はCore側がモデルを保持するため、呼び出し側はFS上のVVMを消してよい。
/// 同じVVMが読み込み済みの場合は何もしない。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voicevox_wasm_load_voice_model(path: *const c_char) -> i32 {
    run(|| {
        let path = unsafe { to_str(path) }?;
        let model = VoiceModelFile::open(path)?;
        synthesizer()?
            .load_voice_model(&model)
            .on_existing(OnExistingVoiceModelId::Skip)
            .perform()?;
        Ok(serde_json::to_vec(model.metas())?)
    })
}

/// 読み込み済みの全VVMのメタ情報をJSONで返す。
#[unsafe(no_mangle)]
pub extern "C" fn voicevox_wasm_metas() -> i32 {
    run(|| Ok(serde_json::to_vec(&synthesizer()?.metas())?))
}

/// `style_id`を含むVVMが読み込み済みなら`1`を、そうでなければ`0`を返す。
///
/// 結果バッファは変更しない。
#[unsafe(no_mangle)]
pub extern "C" fn voicevox_wasm_is_loaded_style(style_id: u32) -> i32 {
    SYNTHESIZER
        .get()
        .is_some_and(|s| s.is_loaded_model_by_style_id(StyleId::new(style_id))) as i32
}

/// テキストからAudioQueryを生成し、JSONで返す。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voicevox_wasm_create_audio_query(
    text: *const c_char,
    style_id: u32,
) -> i32 {
    run(|| {
        let text = unsafe { to_str(text) }?;
        let audio_query = synthesizer()?.create_audio_query(text, StyleId::new(style_id))?;
        Ok(serde_json::to_vec(&audio_query)?)
    })
}

/// テキストからAccentPhraseの配列を生成し、JSONで返す。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voicevox_wasm_create_accent_phrases(
    text: *const c_char,
    style_id: u32,
) -> i32 {
    run(|| {
        let text = unsafe { to_str(text) }?;
        let accent_phrases = synthesizer()?.create_accent_phrases(text, StyleId::new(style_id))?;
        Ok(serde_json::to_vec(&accent_phrases)?)
    })
}

/// AquesTalk風記法からAccentPhraseの配列を生成し、JSONで返す。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voicevox_wasm_create_accent_phrases_from_kana(
    kana: *const c_char,
    style_id: u32,
) -> i32 {
    run(|| {
        let kana = unsafe { to_str(kana) }?;
        let accent_phrases =
            synthesizer()?.create_accent_phrases_from_kana(kana, StyleId::new(style_id))?;
        Ok(serde_json::to_vec(&accent_phrases)?)
    })
}

/// AccentPhraseの配列(JSON)の音高・音素長を生成しなおし、JSONで返す。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voicevox_wasm_replace_mora_data(
    accent_phrases_json: *const c_char,
    style_id: u32,
) -> i32 {
    run(|| {
        let accent_phrases =
            serde_json::from_str::<Vec<AccentPhrase>>(unsafe { to_str(accent_phrases_json) }?)
                .context("invalid accent phrases")?;
        let accent_phrases =
            synthesizer()?.replace_mora_data(&accent_phrases, StyleId::new(style_id))?;
        Ok(serde_json::to_vec(&accent_phrases)?)
    })
}

/// AudioQuery(JSON)から音声合成し、WAVを返す。
///
/// `enable_interrogative_upspeak`が`0`以外なら、疑問文の語尾を自動調整する。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voicevox_wasm_synthesis(
    audio_query_json: *const c_char,
    style_id: u32,
    enable_interrogative_upspeak: i32,
) -> i32 {
    run(|| {
        let audio_query = serde_json::from_str::<AudioQuery>(unsafe { to_str(audio_query_json) }?)
            .context("invalid audio query")?;
        let wav = synthesizer()?
            .synthesis(&audio_query, StyleId::new(style_id))
            .enable_interrogative_upspeak(enable_interrogative_upspeak != 0)
            .perform()?;
        ensure_wav(&wav)?;
        Ok(wav)
    })
}

/// テキストから直接音声合成し、WAVを返す。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voicevox_wasm_tts(text: *const c_char, style_id: u32) -> i32 {
    run(|| {
        let text = unsafe { to_str(text) }?;
        let wav = synthesizer()?.tts(text, StyleId::new(style_id)).perform()?;
        ensure_wav(&wav)?;
        Ok(wav)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn voicevox_wasm_result_pointer() -> *const u8 {
    RESULT.lock().unwrap().as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn voicevox_wasm_result_length() -> usize {
    RESULT.lock().unwrap().len()
}

/// `f`の結果、もしくはエラーメッセージを結果バッファに入れる。
fn run(f: impl FnOnce() -> anyhow::Result<Vec<u8>>) -> i32 {
    let (code, result) = match f() {
        Ok(result) => (0, result),
        Err(error) => {
            let message = format!("{error:#}");
            eprintln!("{message}");
            (1, message.into_bytes())
        }
    };
    *RESULT.lock().unwrap() = result;
    code
}

fn synthesizer() -> anyhow::Result<&'static Synthesizer<OpenJtalk>> {
    SYNTHESIZER
        .get()
        .context("`voicevox_wasm_initialize` has not been completed")
}

/// # Safety
///
/// `ptr`はnullか、NUL終端の文字列を指していなければならない。
unsafe fn to_str<'a>(ptr: *const c_char) -> anyhow::Result<&'a str> {
    ensure!(!ptr.is_null(), "string argument is null");
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .context("string argument is not valid UTF-8")
}

fn ensure_wav(wav: &[u8]) -> anyhow::Result<()> {
    ensure!(
        wav.len() >= 44 && wav.starts_with(b"RIFF") && wav.get(8..12) == Some(b"WAVE"),
        "synthesis did not produce a complete WAV file"
    );
    Ok(())
}

fn main() {}
