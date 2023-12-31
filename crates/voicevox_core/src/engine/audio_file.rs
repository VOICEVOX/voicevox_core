use std::io::{Cursor, Write as _};

use az::{Az as _, Cast};
use num_traits::Float;

use crate::{synthesizer::DEFAULT_SAMPLING_RATE, AudioQueryModel};

pub(crate) fn to_wav<T: Float + From<i16> + From<f32> + Cast<i16>>(
    wave: &[T],
    audio_query: &AudioQueryModel,
) -> Vec<u8> {
    // TODO: ライブラリ(e.g. https://docs.rs/hound)を使う

    let volume_scale = *audio_query.volume_scale();
    let output_stereo = *audio_query.output_stereo();
    let output_sampling_rate = *audio_query.output_sampling_rate();

    // TODO: 44.1kHzなどの対応

    let num_channels: u16 = if output_stereo { 2 } else { 1 };
    let bit_depth: u16 = 16;
    let repeat_count: u32 = (output_sampling_rate / DEFAULT_SAMPLING_RATE) * num_channels as u32;
    let block_size: u16 = bit_depth * num_channels / 8;

    let bytes_size = wave.len() as u32 * repeat_count * 2;
    let wave_size = bytes_size + 44;

    let buf: Vec<u8> = Vec::with_capacity(wave_size as usize);
    let mut cur = Cursor::new(buf);

    cur.write_all("RIFF".as_bytes()).unwrap();
    cur.write_all(&(wave_size - 8).to_le_bytes()).unwrap();
    cur.write_all("WAVEfmt ".as_bytes()).unwrap();
    cur.write_all(&16_u32.to_le_bytes()).unwrap(); // fmt header length
    cur.write_all(&1_u16.to_le_bytes()).unwrap(); //linear PCM
    cur.write_all(&num_channels.to_le_bytes()).unwrap();
    cur.write_all(&output_sampling_rate.to_le_bytes()).unwrap();

    let block_rate = output_sampling_rate * block_size as u32;

    cur.write_all(&block_rate.to_le_bytes()).unwrap();
    cur.write_all(&block_size.to_le_bytes()).unwrap();
    cur.write_all(&bit_depth.to_le_bytes()).unwrap();
    cur.write_all("data".as_bytes()).unwrap();
    cur.write_all(&bytes_size.to_le_bytes()).unwrap();

    for &value in wave {
        let v = num_traits::clamp(
            value * <T as From<_>>::from(volume_scale),
            -T::one(),
            T::one(),
        );
        let data = (v * <T as From<_>>::from(0x7fff)).az::<i16>();
        for _ in 0..repeat_count {
            cur.write_all(&data.to_le_bytes()).unwrap();
        }
    }

    cur.into_inner()
}
