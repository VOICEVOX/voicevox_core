package jp.Hiroshiba.VoicevoxCore;

/** ライブラリを読み込むためだけのクラス。 */
abstract class Dll {
  static {
    String ortPath = System.getenv("ORT_DLL_PATH");
    if (ortPath != null) {
      System.load(ortPath);
    }
    String vvDllPath = System.getenv("VV_DLL_PATH");
    if (vvDllPath != null) {
      System.load(vvDllPath);
    } else {
      System.loadLibrary("voicevox_core_java_api");
    }
  }
}
