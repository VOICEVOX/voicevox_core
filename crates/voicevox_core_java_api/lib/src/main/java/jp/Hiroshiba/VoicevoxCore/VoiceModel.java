package jp.Hiroshiba.VoicevoxCore;

public class VoiceModel
    implements AutoCloseable {
  protected long internal;

  final public String id;
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

  private native String rsGetId();

  private native SpeakerMeta[] rsGetMetas();

  private native void rsDrop();

  static {
    System.loadLibrary("voicevox_core_java_api");
  }

  public static class SpeakerMeta {
    final String name;
    final StyleMeta[] styles;
    final String speakerUuid;
    final String version;

    protected SpeakerMeta(String name, StyleMeta[] styles, String speakerUuid, String version) {
      this.name = name;
      this.styles = styles;
      this.speakerUuid = speakerUuid;
      this.version = version;
    }
  }

  public static class StyleMeta {
    final String name;
    final int id;

    protected StyleMeta(String name, int id) {
      this.name = name;
      this.id = id;
    }
  }
}
