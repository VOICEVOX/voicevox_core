//! テキスト関係やAudioQuery周り、またWAV出力に関する「エンジン」の領域。

mod acoustic_feature_extractor;
mod audio_file;
mod mora_list;
mod ndarray;
mod sampling_rate;
pub(crate) mod song;
pub(crate) mod talk;

pub(crate) use self::{
    acoustic_feature_extractor::PhonemeCode, audio_file::to_s16le_pcm, ndarray::ArrayBase1Ext,
    sampling_rate::DEFAULT_SAMPLING_RATE,
};
pub use self::{
    acoustic_feature_extractor::{Phoneme, Sil},
    audio_file::wav_from_s16le,
};
