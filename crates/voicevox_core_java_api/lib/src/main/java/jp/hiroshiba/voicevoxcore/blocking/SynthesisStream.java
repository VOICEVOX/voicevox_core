package jp.hiroshiba.voicevoxcore.blocking;

import jakarta.annotation.Nonnull;
import java.io.Closeable;
import java.util.NoSuchElementException;
import java.util.function.Consumer;
import jp.hiroshiba.voicevoxcore.exceptions.RunModelException;
import jp.hiroshiba.voicevoxcore.exceptions.StyleNotFoundException;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/**
 * 音声合成を都度行うことで、WAVデータを分割で返すオブジェクト。
 *
 * <p>{@link Synthesizer#streamingSynthesis}により作られる。
 */
public final class SynthesisStream implements Closeable {
  static {
    Dll.loadLibrary();
  }

  private long handle;

  private SynthesisStream() {}

  /**
   * 次の要素があるかどうかを返す。
   *
   * @return 次の要素があるかどうか。
   * @throws IllegalStateException {@link #close()}の後、もしくは{@link #close()}の実行中に呼ばれた場合。
   */
  public boolean hasNext() {
    return rsHasNext();
  }

  /**
   * 次の要素を返す。
   *
   * @return 次の要素。
   * @throws NoSuchElementException 要素が無い場合。
   * @throws StyleNotFoundException 利用中の音声モデルが{@link Synthesizer#unloadVoiceModel アンロード}されていた場合。
   * @throws RunModelException 推論に失敗した、または推論結果が異常だった場合。
   * @throws IllegalStateException {@link #close()}の後、もしくは{@link #close()}の実行中に呼ばれた場合。
   */
  @Nonnull
  public byte[] next() throws RunModelException {
    byte[] chunk = rsNext();
    if (chunk == null) {
      throw new NoSuchElementException();
    }
    return chunk;
  }

  /**
   * 残りの要素をすべて、順に{@code action}へ渡す。
   *
   * @param action 要素を受け取る処理。
   * @throws StyleNotFoundException 利用中の音声モデルが{@link Synthesizer#unloadVoiceModel アンロード}されていた場合。
   * @throws RunModelException 推論に失敗した、または推論結果が異常だった場合。
   * @throws IllegalStateException {@link #close()}の後、もしくは{@link #close()}の実行中に呼ばれた場合。
   */
  public void forEachRemaining(Consumer<byte[]> action) throws RunModelException {
    while (hasNext()) {
      action.accept(next());
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

  private native boolean rsHasNext();

  private native byte[] rsNext() throws RunModelException;

  private native void rsClose();

  private native void rsDrop();
}
