package jp.hiroshiba.voicevoxcore;

import jp.hiroshiba.voicevoxcore.internal.Dll;

/** AudioFeature (音声の中間表現)。 */
public final class AudioFeature {
  static {
    Dll.loadLibrary();
  }

  private long handle;

  protected void finalize() throws Throwable {
    rsDrop();
    super.finalize();
  }

  /** 音声のフレーム数。 */
  public long getFrameLength() {
    return rsGetFrameLength();
  }

  private native long rsGetFrameLength();

  private native void rsDrop();
}
