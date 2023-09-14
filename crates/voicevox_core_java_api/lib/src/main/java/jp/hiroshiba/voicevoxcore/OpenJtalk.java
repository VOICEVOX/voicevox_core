package jp.hiroshiba.voicevoxcore;

public class OpenJtalk extends Dll {
  private long handle;

  /**
   * Open JTalkの辞書ディレクトリ。
   *
   * @param openJtalkDictDir 辞書のディレクトリ。
   */
  public OpenJtalk(String openJtalkDictDir) {
    rsNewWithInitialize(openJtalkDictDir);
  }

  protected void finalize() throws Throwable {
    rsDrop();
    super.finalize();
  }

  /**
   * ユーザー辞書を設定する。
   *
   * <p>この関数を呼び出した後にユーザー辞書を変更した場合は、再度この関数を呼ぶ必要がある。
   *
   * @param userDict ユーザー辞書。
   */
  public void useUserDict(UserDict userDict) {
    rsUseUserDict(userDict);
  }

  private native void rsNewWithoutDic();

  private native void rsNewWithInitialize(String openJtalkDictDir);

  private native void rsUseUserDict(UserDict userDict);

  private native void rsDrop();
}
