//! テキスト関係やAudioQuery周り、またWAV出力に関する「エンジン」の領域。

mod acoustic_feature_extractor;
mod audio_file;
mod fundamental;
mod mora_list;
pub(crate) mod song;
pub(crate) mod talk;

pub use self::audio_file::wav_from_s16le;
pub(crate) use self::{
    acoustic_feature_extractor::{Phoneme, PhonemeCode},
    audio_file::to_s16le_pcm,
    fundamental::DEFAULT_SAMPLING_RATE,
};
