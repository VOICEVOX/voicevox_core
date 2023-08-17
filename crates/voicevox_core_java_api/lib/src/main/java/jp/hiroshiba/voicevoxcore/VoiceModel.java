package jp.hiroshiba.voicevoxcore;

import com.google.gson.Gson;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import java.lang.ref.Cleaner;
import javax.annotation.Nonnull;

/** 音声モデル。 */
public class VoiceModel extends Dll {
  private long handle;
  private static final Cleaner cleaner = Cleaner.create();

  /** ID。 */
  @Nonnull public final String id;

  /** メタ情報。 */
  @Nonnull public final SpeakerMeta[] metas;

  public VoiceModel(String modelPath) {
    rsFromPath(modelPath);
    id = rsGetId();
    String metasJson = rsGetMetasJson();
    Gson gson = new Gson();
    SpeakerMeta[] rawMetas = gson.fromJson(metasJson, SpeakerMeta[].class);
    if (rawMetas == null) {
      throw new RuntimeException("Failed to parse metasJson");
    }
    metas = rawMetas;

    cleaner.register(this, () -> rsDrop());
  }

  private native void rsFromPath(String modelPath);

  @Nonnull
  private native String rsGetId();

  @Nonnull
  private native String rsGetMetasJson();

  private native void rsDrop();

  /** 話者（speaker）のメタ情報。 */
  public static class SpeakerMeta {
    /** 話者名。 */
    @SerializedName("name")
    @Expose
    @Nonnull
    final String name;

    /** 話者に属するスタイル。 */
    @SerializedName("styles")
    @Expose
    @Nonnull
    final StyleMeta[] styles;

    /** 話者のUUID。 */
    @SerializedName("speaker_uuid")
    @Expose
    @Nonnull
    final String speakerUuid;

    /** 話者のバージョン。 */
    @SerializedName("version")
    @Expose
    @Nonnull
    final String version;

    private SpeakerMeta() {
      // GSONからコンストラクトするため、このメソッドは呼ばれることは無い。
      // このメソッドは@Nonnullを満たすために必要。
      this.name = "";
      this.styles = new StyleMeta[0];
      this.speakerUuid = "";
      this.version = "";
    }
  }

  /** スタイル（style）のメタ情報。 */
  public static class StyleMeta {
    /** スタイル名。 */
    @SerializedName("name")
    @Expose
    @Nonnull
    final String name;

    /** スタイルID。 */
    @SerializedName("id")
    @Expose
    final int id;

    private StyleMeta() {
      this.name = "";
      this.id = 0;
    }
  }
}
