package jp.hiroshiba.voicevoxcore;

import jp.hiroshiba.voicevoxcore.internal.Dll;

public final class Frames {
  static {
    Dll.loadLibrary();
  }

  /**
   * フレームレート。
   *
   * <p>音声の秒数は<code>{@link AudioQuery#frameLength() frameLength} / FRAME_RATE</code>で表せる。
   *
   * <p><strong>Caveats</strong>: この定数は将来的に削除される可能性がある。例えば、<code>{@link
   * StyleMeta}#frameRate</code>というフィールドに置き換えられる可能性がある。
   */
  public static final double FRAME_RATE = 93.75;

  static {
    assert FRAME_RATE == rsFrameRate();
  }

  private static native double rsFrameRate();

  private Frames() {}
}
