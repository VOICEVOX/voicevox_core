use std::sync::OnceLock;

use anyhow::ensure;
use voicevox_core::{
    AccelerationMode, StyleId,
    blocking::{OpenJtalk, Synthesizer, VoiceModelFile},
};

static SYNTHESIZED_WAV: OnceLock<Box<[u8]>> = OnceLock::new();

#[unsafe(no_mangle)]
pub extern "C" fn voicevox_browser_synthesize() -> i32 {
    match synthesize() {
        Ok(wav) => match SYNTHESIZED_WAV.set(wav) {
            Ok(()) => 0,
            Err(_) => {
                eprintln!("browser smoke synthesis was already completed");
                1
            }
        },
        Err(error) => {
            eprintln!("{error:#}");
            1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn voicevox_browser_wav_pointer() -> *const u8 {
    SYNTHESIZED_WAV
        .get()
        .map_or(std::ptr::null(), |wav| wav.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn voicevox_browser_wav_length() -> usize {
    SYNTHESIZED_WAV.get().map_or(0, |wav| wav.len())
}

fn synthesize() -> anyhow::Result<Box<[u8]>> {
    let runtime = voicevox_core::blocking::Onnxruntime::init_once()?;
    let synthesizer = Synthesizer::builder(runtime)
        .text_analyzer(OpenJtalk::new("")?)
        .acceleration_mode(AccelerationMode::Cpu)
        .cpu_num_threads(1)
        .build()?;
    let voice_model = VoiceModelFile::open("/sample.vvm")?;
    synthesizer.load_voice_model(&voice_model).perform()?;
    let wav = synthesizer
        .tts("これはテストです", StyleId::new(302))
        .perform()?;
    ensure!(
        wav.len() >= 44 && wav.starts_with(b"RIFF") && wav.get(8..12) == Some(b"WAVE"),
        "synthesis did not produce a complete WAV file"
    );
    Ok(wav.into_boxed_slice())
}

fn main() {}
