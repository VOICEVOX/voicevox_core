package jp.Hiroshiba.VoicevoxCore;

import javax.annotation.Nonnull;

import com.google.gson.Gson;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;

/** 音声モデル。 */
public class VoiceModel extends Dll implements AutoCloseable {
  long internal;

  /** ID。 */
  @Nonnull
  final public String id;
  /** メタ情報。 */
  @Nonnull
  final public SpeakerMeta[] metas;

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

  }

  /**
   * 音声モデルを廃棄する。
   */
  public void close() {
    rsDrop();
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
