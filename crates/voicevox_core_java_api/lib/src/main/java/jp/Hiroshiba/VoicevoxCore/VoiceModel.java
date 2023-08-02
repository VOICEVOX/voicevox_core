package jp.Hiroshiba.VoicevoxCore;

import javax.annotation.Nonnull;

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
    metas = rsGetMetas();
  }

  public void close() {
    rsDrop();
  }

  private native void rsFromPath(String modelPath);

  @Nonnull
  private native String rsGetId();

  @Nonnull
  private native SpeakerMeta[] rsGetMetas();

  private native void rsDrop();

  static {
    System.loadLibrary("voicevox_core_java_api");
  }

  public static class SpeakerMeta {
    @Nonnull
    final String name;
    @Nonnull
    final StyleMeta[] styles;
    @Nonnull
    final String speakerUuid;
    @Nonnull
    final String version;

    protected SpeakerMeta(String name, StyleMeta[] styles, String speakerUuid, String version) {
      if (name == null) {
        throw new NullPointerException("name");
      }
      if (styles == null) {
        throw new NullPointerException("styles");
      }
      if (speakerUuid == null) {
        throw new NullPointerException("speakerUuid");
      }
      if (version == null) {
        throw new NullPointerException("version");
      }
      this.name = name;
      this.styles = styles;
      this.speakerUuid = speakerUuid;
      this.version = version;
    }
  }

  public static class StyleMeta {
    @Nonnull
    final String name;
    final int id;

    protected StyleMeta(String name, int id) {
      if (name == null) {
        throw new NullPointerException("name");
      }
      this.name = name;
      this.id = id;
    }
  }
}
