package jp.hiroshiba.voicevoxcore.blocking;

import com.google.gson.Gson;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import jp.hiroshiba.voicevoxcore.AccentPhrase;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/** テキスト解析機としてのOpen JTalk。 */
public class OpenJtalk {
  static {
    Dll.loadLibrary();
  }

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

  public List<AccentPhrase> analyze(String text) {
    Gson gson = new Gson();
    String accentPhrasesJson = rsAnalyze(text);
    AccentPhrase[] rawAccentPhrases = gson.fromJson(accentPhrasesJson, AccentPhrase[].class);
    if (rawAccentPhrases == null) {
      throw new NullPointerException("accent_phrases");
    }
    return new ArrayList<AccentPhrase>(Arrays.asList(rawAccentPhrases));
  }

  private native void rsNew(String openJtalkDictDir);

  private native void rsUseUserDict(UserDict userDict);

  private native String rsAnalyze(String text);

  private native void rsDrop();
}
