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
#[cfg_attr(
    doc,
    doc(alias = "VOICEVOX_FRAME_RATE", alias = "voicevox_get_frame_rate")
)]
pub const FRAME_RATE: f64 = 93.75;

/// １フレームから生成されるPCMのサンプル数。
#[cfg_attr(
    doc,
    doc(
        alias = "VOICEVOX_WAVE_SAMPLES_PER_FRAME",
        alias = "voicevox_get_wave_samples_per_frame"
    )
)]
pub const WAVE_SAMPLES_PER_FRAME: u16 = 256;
