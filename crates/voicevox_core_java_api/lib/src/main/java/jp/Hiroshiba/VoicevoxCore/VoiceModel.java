package jp.Hiroshiba.VoicevoxCore;

import javax.annotation.Nonnull;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.gson.Gson;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;

/** 音声モデル。 */
public class VoiceModel
    implements AutoCloseable {
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

  static {
    System.loadLibrary("voicevox_core_java_api");
  }

  /** 話者（speaker）のメタ情報。 */
  public static class SpeakerMeta {
    /** 話者名。 */
    @JsonProperty("name")
    @SerializedName("name")
    @Expose
    @Nonnull
    final String name;
    /** 話者に属するスタイル。 */
    @JsonProperty("styles")
    @SerializedName("styles")
    @Expose
    @Nonnull
    final StyleMeta[] styles;
    /** 話者のUUID。 */
    @JsonProperty("speaker_uuid")
    @SerializedName("speaker_uuid")
    @Expose
    @Nonnull
    final String speakerUuid;
    /** 話者のバージョン。 */
    @JsonProperty("version")
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
    @JsonProperty("name")
    @SerializedName("name")
    @Expose
    @Nonnull
    final String name;
    /** スタイルID。 */
    @JsonProperty("id")
    @SerializedName("id")
    @Expose
    final int id;

    private StyleMeta() {
      this.name = "";
      this.id = 0;
    }
  }
}
