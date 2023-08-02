package jp.Hiroshiba.VoicevoxCore;

import javax.annotation.Nonnull;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.gson.Gson;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;

public class VoiceModel
    implements AutoCloseable {
  protected long internal;

  @Nonnull
  final public String id;
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

  public static class SpeakerMeta {
    @JsonProperty("name")
    @SerializedName("name")
    @Expose
    @Nonnull
    final String name;
    @JsonProperty("styles")
    @SerializedName("styles")
    @Expose
    @Nonnull
    final StyleMeta[] styles;
    @JsonProperty("speaker_uuid")
    @SerializedName("speaker_uuid")
    @Expose
    @Nonnull
    final String speakerUuid;
    @JsonProperty("version")
    @SerializedName("version")
    @Expose
    @Nonnull
    final String version;

    private SpeakerMeta() {
      this.name = "";
      this.styles = new StyleMeta[0];
      this.speakerUuid = "";
      this.version = "";
    }
  }

  public static class StyleMeta {
    @JsonProperty("name")
    @SerializedName("name")
    @Expose
    @Nonnull
    final String name;
    @JsonProperty("id")
    @SerializedName("id")
    @Expose
    final int id;

    private StyleMeta() {
      // GSONからコンストラクトするため、このメソッドは呼ばれることは無い。
      // このメソッドは@Nonnullを満たすために必要。
      this.name = "";
      this.id = 0;
    }
  }
}
