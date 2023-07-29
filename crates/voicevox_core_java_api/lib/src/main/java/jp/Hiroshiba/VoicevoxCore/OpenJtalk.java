package jp.Hiroshiba.VoicevoxCore;

public class OpenJtalk {
  private long internalPtr;

  public OpenJtalk() {
    internalPtr = rsNewWithoutDic();
  }

  public OpenJtalk(String open_jtalk_dict_dir) {
    internalPtr = rsNewWithInitilize(open_jtalk_dict_dir);
  }

  private native long rsNewWithoutDic();

  private native long rsNewWithInitilize(String open_jtalk_dict_dir);

  static {
    System.loadLibrary("voicevox_core_java_api");
  }
}
