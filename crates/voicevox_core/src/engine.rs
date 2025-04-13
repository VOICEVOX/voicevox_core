//! テキスト関係やAudioQuery周り、またWAV出力に関する「エンジン」の領域。

mod audio_file;
pub(crate) mod talk;

pub(crate) use self::audio_file::to_s16le_pcm;
pub use self::audio_file::wav_from_s16le;
