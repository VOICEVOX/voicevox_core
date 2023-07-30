package jp.Hiroshiba.VoicevoxCore;

public class OpenJtalk {
  protected long internalPtr;

  public OpenJtalk() {
    rsNewWithoutDic();
  }

  public OpenJtalk(String openJtalkDictDir) {
    rsNewWithInitilize(openJtalkDictDir);
  }

  private native boolean rsNewWithoutDic();

  private native boolean rsNewWithInitilize(String openJtalkDictDir);

  static {
    System.loadLibrary("voicevox_core_java_api");
  }
}
