/// フレームレート。
///
/// 音声の秒数は<code>[frame_length] / FRAME_RATE</code>で表せる。
///
/// # Caveats
///
/// この定数は将来的に削除される可能性がある。例えば、<code>[StyleMeta]::frame_rate</code>というフィールドに置き換えられる可能性がある。
///
/// [frame_length]: crate::AudioQuery::frame_length
/// [StyleMeta]: crate::StyleMeta
pub const FRAME_RATE: f64 = 93.75;

/// １フレームから生成されるPCMのサンプル数。
pub const WAVE_SAMPLES_PER_FRAME: u16 = 256;
