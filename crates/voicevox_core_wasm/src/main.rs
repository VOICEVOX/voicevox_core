use std::{
    ffi::{CStr, c_char},
    sync::{Mutex, OnceLock},
};

use anyhow::{Context as _, ensure};
use voicevox_core::{
    AccelerationMode, StyleId,
    blocking::{OpenJtalk, Synthesizer, VoiceModelFile},
};

const OPEN_JTALK_DICT_DIR: &str = "/open_jtalk_dic_utf_8-1.11";
const VOICE_MODEL_PATH: &str = "/sample.vvm";

static SYNTHESIZER: OnceLock<Synthesizer<OpenJtalk>> = OnceLock::new();
static SYNTHESIZED_WAV: Mutex<Option<Box<[u8]>>> = Mutex::new(None);

/// 辞書と音声モデルをEmscripten FSに書き込んだ後、一度だけ呼ぶ。
#[unsafe(no_mangle)]
pub extern "C" fn voicevox_browser_initialize() -> i32 {
    if SYNTHESIZER.get().is_some() {
        return 0;
    }
    match initialize() {
        Ok(synthesizer) => {
            let _ = SYNTHESIZER.set(synthesizer);
            0
        }
        Err(error) => {
            eprintln!("{error:#}");
            1
        }
    }
}

/// `text`はNUL終端のUTF-8文字列。成功した場合、WAVは次の呼び出しまで保持される。
#[unsafe(no_mangle)]
pub extern "C" fn voicevox_browser_synthesize(text: *const c_char, style_id: u32) -> i32 {
    // SAFETY: 呼び出し側がNUL終端の文字列を渡す
    match unsafe { synthesize(text, style_id) } {
        Ok(wav) => {
            *SYNTHESIZED_WAV.lock().unwrap() = Some(wav);
            0
        }
        Err(error) => {
            eprintln!("{error:#}");
            1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn voicevox_browser_wav_pointer() -> *const u8 {
    SYNTHESIZED_WAV
        .lock()
        .unwrap()
        .as_ref()
        .map_or(std::ptr::null(), |wav| wav.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn voicevox_browser_wav_length() -> usize {
    SYNTHESIZED_WAV
        .lock()
        .unwrap()
        .as_ref()
        .map_or(0, |wav| wav.len())
}

fn initialize() -> anyhow::Result<Synthesizer<OpenJtalk>> {
    let runtime = voicevox_core::blocking::Onnxruntime::init_once()?;
    let synthesizer = Synthesizer::builder(runtime)
        .text_analyzer(OpenJtalk::new(OPEN_JTALK_DICT_DIR)?)
        .acceleration_mode(AccelerationMode::Cpu)
        .cpu_num_threads(1)
        .build()?;
    let voice_model = VoiceModelFile::open(VOICE_MODEL_PATH)?;
    synthesizer.load_voice_model(&voice_model).perform()?;
    Ok(synthesizer)
}

unsafe fn synthesize(text: *const c_char, style_id: u32) -> anyhow::Result<Box<[u8]>> {
    let synthesizer = SYNTHESIZER
        .get()
        .context("`voicevox_browser_initialize` has not been completed")?;
    ensure!(!text.is_null(), "text is null");
    let text = unsafe { CStr::from_ptr(text) }
        .to_str()
        .context("text is not valid UTF-8")?;
    let wav = synthesizer.tts(text, StyleId::new(style_id)).perform()?;
    ensure!(
        wav.len() >= 44 && wav.starts_with(b"RIFF") && wav.get(8..12) == Some(b"WAVE"),
        "synthesis did not produce a complete WAV file"
    );
    Ok(wav.into_boxed_slice())
}

fn main() {}
