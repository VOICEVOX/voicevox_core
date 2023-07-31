package jp.Hiroshiba.VoicevoxCore;

public class OpenJtalk implements AutoCloseable {
  protected long internal;

  public OpenJtalk() {
    rsNewWithoutDic();
  }

  public OpenJtalk(String openJtalkDictDir) {
    rsNewWithInitilize(openJtalkDictDir);
  }

  public void close() {
    rsDrop();
  }

  private native void rsNewWithoutDic();

  private native void rsNewWithInitilize(String openJtalkDictDir);

  private native void rsDrop();

  static {
    System.loadLibrary("voicevox_core_java_api");
  }
}
