package jp.Hiroshiba.VoicevoxCore;

/** テキスト解析器としてのOpen JTalk。 */
public class OpenJtalk implements AutoCloseable {
  protected long internal;

  /**
   * Open JTalkの辞書ディレクトリ。
   *
   * @param openJtalkDictDir 辞書のディレクトリ。
   */
  public OpenJtalk(String openJtalkDictDir) {
    rsNewWithInitialize(openJtalkDictDir);
  }

  /**
   * ユーザー辞書を設定する。
   *
   * この関数を呼び出した後にユーザー辞書を変更した場合は、再度この関数を呼ぶ必要がある。
   *
   * @param userDict ユーザー辞書。
   */
  public void useUserDict(UserDict userDict) {
    rsUseUserDict(userDict);
  }

  /** Open JTalkを廃棄する。 */
  public void close() {
    rsDrop();
  }

  private native void rsNewWithoutDic();

  private native void rsNewWithInitialize(String openJtalkDictDir);

  private native void rsUseUserDict(UserDict userDict);

  private native void rsDrop();

  static {
    System.loadLibrary("voicevox_core_java_api");
  }
}
