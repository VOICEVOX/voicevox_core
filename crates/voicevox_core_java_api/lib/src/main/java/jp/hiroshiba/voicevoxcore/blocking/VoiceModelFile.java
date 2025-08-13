package jp.hiroshiba.voicevoxcore.blocking;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import jakarta.annotation.Nonnull;
import java.io.Closeable;
import java.util.UUID;
import jp.hiroshiba.voicevoxcore.CharacterMeta;
import jp.hiroshiba.voicevoxcore.StyleType;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/** 音声モデルファイル。 */
public class VoiceModelFile implements Closeable {
  static {
    Dll.loadLibrary();
  }

  private long handle;

  /**
   * ID。
   *
   * <p>{@link Synthesizer}はこのIDをキーとして、音声モデルのロード・アンロードを行う。
   *
   * <p>同じIDを持つ複数のVVMファイルがあるときは、ファイルとして新しい方を常に使うことが推奨される。<a
   * href="https://github.com/VOICEVOX/voicevox_vvm">VOICEVOX/voicevox_vvm</a>で管理されているVVMでは、次のような方針が取られている。
   *
   * <ul>
   *   <li>VVMに含まれる声が変化せず、軽微な修正のみのときはIDを使い回してリリースする。
   *   <li>VVMに含まれる声が明確に変化するかもしくは削除されるような実質的な変更のときは、新しいIDを割り振ってリリースする。
   * </ul>
   *
   * <p>これ以外は未定であり、更なるルールについては<a
   * href="https://github.com/VOICEVOX/voicevox_vvm/issues/19">VOICEVOX/voicevox_vvm#19</a>で議論される予定。
   *
   * <p>{@link #close}の後でも利用可能。
   */
  @Nonnull public final UUID id;

  /**
   * メタ情報。
   *
   * <p>{@link #close}の後でも利用可能。
   */
  @Nonnull public final CharacterMeta[] metas;

  public VoiceModelFile(String modelPath) {
    rsOpen(modelPath);
    id = rsGetId();
    String metasJson = rsGetMetasJson();
    GsonBuilder gsonBuilder = new GsonBuilder();
    gsonBuilder.registerTypeAdapter(StyleType.class, new StyleType.Deserializer());
    Gson gson = gsonBuilder.create();
    CharacterMeta[] rawMetas = gson.fromJson(metasJson, CharacterMeta[].class);
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
