package jp.hiroshiba.voicevoxcore.blocking;

import jakarta.annotation.Nullable;
import java.io.Closeable;
import java.util.function.Consumer;
import jp.hiroshiba.voicevoxcore.exceptions.RunModelException;
import jp.hiroshiba.voicevoxcore.exceptions.StyleNotFoundException;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/**
 * ストリーミング音声合成の結果。
 *
 * <p>{@link #next()}を呼ぶたびに、音声の断片(最初はWAVヘッダー、その後は16-bit PCM)が1つずつ合成される。{@link
 * #next()}を呼ばない限り、音声合成は実行されない。
 *
 * @see Synthesizer#streamingSynthesis
 */
public final class SynthesisStream implements Closeable {
  static {
    Dll.loadLibrary();
  }

  private long handle;

  private SynthesisStream() {}

  /**
   * 次の音声の断片を合成して返す。
   *
   * <p>最初の要素は必ず、WAVデータの先頭部分を示す44バイトのデータになる。このときエラーが返されることはない。
   *
   * <p>複数のスレッドから同時に呼んだ場合は、順番に実行される。
   *
   * @return 音声の断片。すべて返し終えた後は{@code null}。
   * @throws RunModelException 推論に失敗した場合。
   * @throws StyleNotFoundException 利用中の音声モデルがアンロードされていた場合。
   * @throws IllegalStateException {@link #close()}の後、もしくは{@link #close()}の実行中に呼ばれた場合。
   */
  @Nullable
  public byte[] next() throws RunModelException {
    return rsNext();
  }

  /**
   * 残りの音声の断片をすべて合成し、順に{@code action}へ渡す。
   *
   * @param action 音声の断片を受け取る処理。
   * @throws RunModelException 推論に失敗した場合。
   * @throws StyleNotFoundException 利用中の音声モデルがアンロードされていた場合。
   * @throws IllegalStateException {@link #close()}の後、もしくは{@link #close()}の実行中に呼ばれた場合。
   */
  public void forEachRemaining(Consumer<byte[]> action) throws RunModelException {
    for (byte[] chunk = next(); chunk != null; chunk = next()) {
      action.accept(chunk);
    }
  }

  /**
   * 内部のリソースを解放する。複数回呼んでもよい。
   *
   * <p>このメソッドが呼ばれた段階で{@link #next()}が実行中の場合、実行が終わるまで待つ。
   */
  @Override
  public void close() {
    rsClose();
  }

  @Override
  protected void finalize() throws Throwable {
    rsDrop();
    super.finalize();
  }

  @Nullable
  private native byte[] rsNext() throws RunModelException;

  private native void rsClose();

  private native void rsDrop();
}
