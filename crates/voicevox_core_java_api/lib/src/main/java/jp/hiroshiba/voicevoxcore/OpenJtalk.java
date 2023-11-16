package jp.hiroshiba.voicevoxcore;

/** テキスト解析機としてのOpen JTalk。 */
public class OpenJtalk extends Dll {
  private long handle;

  /**
   * Open JTalkの辞書ディレクトリ。
   *
   * @param openJtalkDictDir 辞書のディレクトリ。
   */
  public OpenJtalk(String openJtalkDictDir) {
    rsNew(openJtalkDictDir);
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
   * <p>Androidの場合、この関数を呼び出す前に {@code TMPDIR} 環境変数を設定する必要がある。
   *
   * <pre>{@code
   * Activity activity = ...;
   * Os.setenv("TMPDIR", activity.getCacheDir().getAbsolutePath(), true);
   * }</pre>
   *
   * @param userDict ユーザー辞書。
   */
  public void useUserDict(UserDict userDict) {
    rsUseUserDict(userDict);
  }

  private native void rsNew(String openJtalkDictDir);

  private native void rsUseUserDict(UserDict userDict);

  private native void rsDrop();
}
