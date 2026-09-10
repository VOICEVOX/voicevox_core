package jp.hiroshiba.voicevoxcore;

import jp.hiroshiba.voicevoxcore.internal.Dll;

/** AudioFeature (音声の中間表現)。 */
public final class AudioFeature {
  static {
    Dll.loadLibrary();
  }

  /** フレームレート。全体の秒数は{@code getFrameLength() / FRAME_RATE}で表せる。 */
  public static final double FRAME_RATE = 24000. / 256.;

  static {
    assert FRAME_RATE == rsFrameRate();
  }

  private static native double rsFrameRate();

  private long handle;

  private AudioFeature(long handle) {
    this.handle = handle;
  }

  protected void finalize() throws Throwable {
    rsDrop();
    super.finalize();
  }

  /**
   * 音声のフレーム数を取得する。
   *
   * @return 音声のフレーム数。
   */
  public long getFrameLength() {
    return rsGetFrameLength();
  }

  private native long rsGetFrameLength();

  private native void rsDrop();
}
