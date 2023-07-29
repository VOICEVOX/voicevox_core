package jp.Hiroshiba.VoicevoxCore;

public class OpenJtalk {
  native static boolean test();

  static {
    System.loadLibrary("voicevox_core_java_api");
  }
}
