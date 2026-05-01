package jp.hiroshiba.voicevoxcore;

import jp.hiroshiba.voicevoxcore.exceptions.IncompatibleQueriesException;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidQueryException;
import jp.hiroshiba.voicevoxcore.internal.Dll;

public final class Queries {
  static {
    Dll.loadLibrary();
  }

  private Queries() {}

  /**
   * 与えられた楽譜と歌唱音声合成用のクエリの組み合わせが、基本周波数と音量の生成に利用できるかどうかを確認する。
   *
   * @param score 楽譜
   * @param frameAudioQuery 歌唱音声合成用のクエリ
   * @throws InvalidQueryException 次のうちどれかを満たす場合
   *     <ul>
   *       <li>{@code score}に対する{@link Score#validate}が失敗する。
   *       <li>{@code frameAudioQuery}に対する{@link FrameAudioQuery#validate}が失敗する。
   *     </ul>
   *
   * @throws IncompatibleQueriesException 次を満たす場合
   *     <ul>
   *       <li>{@code score}が表す音素ID列と、{@code
   *           frameAudioQuery}が表す音素ID列が等しくない。ただし異なる音素の表現が同一のIDを表すことがある。
   *     </ul>
   */
  public static void ensureCompatible(Score score, FrameAudioQuery frameAudioQuery) {
    rsEnsureCompatible(score, frameAudioQuery);
  }

  private static native void rsEnsureCompatible(Score score, FrameAudioQuery frameAudioQuery);
}
