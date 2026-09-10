package jp.hiroshiba.voicevoxcore;

import jp.hiroshiba.voicevoxcore.internal.Dll;

/** WAVフォーマットに関するユーティリティ。 */
public final class Wav {
  static {
    Dll.loadLibrary();
  }

  public static byte[] wavFromS16le(byte[] pcm, int sampleRate, boolean stereo) {
    return rsWavFromS16le(pcm, sampleRate, stereo);
  }

  private static native byte[] rsWavFromS16le(byte[] pcm, int sampleRate, boolean stereo);
}
