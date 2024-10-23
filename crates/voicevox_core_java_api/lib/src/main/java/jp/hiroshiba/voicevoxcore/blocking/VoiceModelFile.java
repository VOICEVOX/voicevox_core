package jp.hiroshiba.voicevoxcore.blocking;

import com.google.gson.Gson;
import jakarta.annotation.Nonnull;
import java.io.Closeable;
import java.util.UUID;
import jp.hiroshiba.voicevoxcore.SpeakerMeta;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/** 音声モデルファイル。 */
public class VoiceModelFile implements Closeable {
  static {
    Dll.loadLibrary();
  }

  private long handle;

  /** ID。 */
  @Nonnull public final UUID id;

  /** メタ情報。 */
  @Nonnull public final SpeakerMeta[] metas;

  public VoiceModelFile(String modelPath) {
    rsOpen(modelPath);
    id = rsGetId();
    String metasJson = rsGetMetasJson();
    Gson gson = new Gson();
    SpeakerMeta[] rawMetas = gson.fromJson(metasJson, SpeakerMeta[].class);
    if (rawMetas == null) {
      throw new RuntimeException("Failed to parse metasJson");
    }
    metas = rawMetas;
  }

  /**
   * VVMファイルを閉じる。
   *
   * <p>このメソッドが呼ばれた段階で{@link Synthesizer#loadVoiceModel}からのアクセスが継続中の場合、アクセスが終わるまで待つ。
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

  private native void rsOpen(String modelPath);

  @Nonnull
  private native UUID rsGetId();

  @Nonnull
  private native String rsGetMetasJson();

  private native void rsClose();

  private native void rsDrop();
}
