package jp.Hiroshiba.VoicevoxCore;

public class VoiceModel
    implements AutoCloseable {
  protected long internal;

  public String id;
  public SpeakerMeta[] metas;

  public VoiceModel(String modelPath) {
    rsFromPath(modelPath);
  }

  public void close() {
    rsDrop();
  }

  private native void rsFromPath(String modelPath);

  private native void rsDrop();

  static {
    System.loadLibrary("voicevox_core_java_api");
  }

  public static class SpeakerMeta {
    String name;
    StyleMeta[] styles;
    String speakerUuid;
    String version;
  }

  public static class StyleMeta {
    String name;
    int id;
  }
}
