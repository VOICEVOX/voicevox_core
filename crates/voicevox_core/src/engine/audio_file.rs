use std::io::{Cursor, Write as _};

/// 16bit PCMにヘッダを付加しWAVフォーマットのバイナリを生成する。
pub fn wav_from_s16le(pcm: &[u8], output_sampling_rate: u32, output_stereo: bool) -> Vec<u8> {
    // TODO: 44.1kHzなどの対応

    let num_channels: u16 = if output_stereo { 2 } else { 1 };
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
    cur.write_all(&1_u16.to_le_bytes()).unwrap(); //linear PCM
    cur.write_all(&num_channels.to_le_bytes()).unwrap();
    cur.write_all(&output_sampling_rate.to_le_bytes()).unwrap();

    let block_rate = output_sampling_rate * block_size as u32;

    cur.write_all(&block_rate.to_le_bytes()).unwrap();
    cur.write_all(&block_size.to_le_bytes()).unwrap();
    cur.write_all(&bit_depth.to_le_bytes()).unwrap();
    cur.write_all("data".as_bytes()).unwrap();
    cur.write_all(&bytes_size.to_le_bytes()).unwrap();
    cur.write_all(&pcm).unwrap();
    cur.into_inner()
}
