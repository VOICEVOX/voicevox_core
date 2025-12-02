use std::io::{Cursor, Write as _};

use crate::{FrameAudioQuery, SamplingRate};

use super::{talk::ValidatedAudioQuery, DEFAULT_SAMPLING_RATE};

pub(crate) fn to_s16le_pcm(wave: &[f32], query: &impl HasPcmOptions) -> Vec<u8> {
    let PcmOptions {
        volume_scale,
        output_sampling_rate,
        output_stereo,
    } = query.pcm_options();
    let num_channels: u16 = if output_stereo { 2 } else { 1 };
    let repeat_count: u32 =
        (output_sampling_rate.get() / DEFAULT_SAMPLING_RATE) * num_channels as u32;
    let bytes_size = wave.len() as u32 * repeat_count * 2;
    let buf: Vec<u8> = Vec::with_capacity(bytes_size as usize);
    let mut cur = Cursor::new(buf);

    for value in wave {
        let v = (value * volume_scale).clamp(-1., 1.);
        let data = (v * 0x7fff as f32) as i16;
        for _ in 0..repeat_count {
            cur.write_all(&data.to_le_bytes()).unwrap();
        }
    }

    cur.into_inner()
}

pub(crate) struct PcmOptions {
    volume_scale: f32,
    output_sampling_rate: SamplingRate,
    output_stereo: bool,
}

pub(crate) trait HasPcmOptions {
    fn pcm_options(&self) -> PcmOptions;
}

impl HasPcmOptions for ValidatedAudioQuery<'_> {
    fn pcm_options(&self) -> PcmOptions {
        let Self {
            volume_scale,
            output_sampling_rate,
            output_stereo,
            ..
        } = *self;

        PcmOptions {
            volume_scale,
            output_sampling_rate,
            output_stereo,
        }
    }
}

impl HasPcmOptions for FrameAudioQuery {
    fn pcm_options(&self) -> PcmOptions {
        let Self {
            volume_scale,
            output_sampling_rate,
            output_stereo,
            ..
        } = *self;

        PcmOptions {
            volume_scale: volume_scale.into(),
            output_sampling_rate,
            output_stereo,
        }
    }
}

/// 16bit PCMにヘッダを付加しWAVフォーマットのバイナリを生成する。
// TODO: 後で復活させる
// https://github.com/VOICEVOX/voicevox_core/issues/970
#[doc(hidden)]
pub fn wav_from_s16le(pcm: &[u8], sampling_rate: u32, is_stereo: bool) -> Vec<u8> {
    let num_channels: u16 = if is_stereo { 2 } else { 1 };
    let bit_depth: u16 = 16;
    let block_size: u16 = bit_depth * num_channels / 8;

    let bytes_size = pcm.len() as u32;
    let wave_size = bytes_size + 44;

    let buf: Vec<u8> = Vec::with_capacity(wave_size as usize);
    let mut cur = Cursor::new(buf);

    cur.write_all("RIFF".as_bytes()).unwrap();
    cur.write_all(&(wave_size - 8).to_le_bytes()).unwrap();
    cur.write_all("WAVEfmt ".as_bytes()).unwrap();
    cur.write_all(&16_u32.to_le_bytes()).unwrap(); // fmt header length
    cur.write_all(&1_u16.to_le_bytes()).unwrap(); // linear PCM
    cur.write_all(&num_channels.to_le_bytes()).unwrap();
    cur.write_all(&sampling_rate.to_le_bytes()).unwrap();

    let block_rate = sampling_rate * block_size as u32;

    cur.write_all(&block_rate.to_le_bytes()).unwrap();
    cur.write_all(&block_size.to_le_bytes()).unwrap();
    cur.write_all(&bit_depth.to_le_bytes()).unwrap();
    cur.write_all("data".as_bytes()).unwrap();
    cur.write_all(&bytes_size.to_le_bytes()).unwrap();
    cur.write_all(pcm).unwrap();
    cur.into_inner()
}
