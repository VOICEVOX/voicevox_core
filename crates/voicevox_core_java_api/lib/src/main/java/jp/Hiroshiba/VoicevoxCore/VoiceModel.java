package jp.Hiroshiba.VoicevoxCore;

public class VoiceModel {
  protected long internalPtr;

  public String id;
  public SpeakerMeta[] metas;

  public VoiceModel(String modelPath) {
    internalPtr = rsFromPath(modelPath);
  }

  native static long rsFromPath(String modelPath);

  static {
    System.loadLibrary("voicevox_core_java_api");
  }
}
