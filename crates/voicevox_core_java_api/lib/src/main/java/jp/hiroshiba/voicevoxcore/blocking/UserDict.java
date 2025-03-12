package jp.hiroshiba.voicevoxcore.blocking;

import jakarta.annotation.Nonnull;
import java.io.File;
import java.nio.file.Path;
import java.util.HashMap;
import jp.hiroshiba.voicevoxcore.UserDictWord;
import jp.hiroshiba.voicevoxcore.exceptions.LoadUserDictException;
import jp.hiroshiba.voicevoxcore.exceptions.SaveUserDictException;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/** ユーザー辞書。 */
public class UserDict {
  static {
    Dll.loadLibrary();
  }

  private long handle;

  /** ユーザー辞書を作成する。 */
  public UserDict() {
    rsNew();
  }

  protected void finalize() throws Throwable {
    rsDrop();
    super.finalize();
  }

  /**
   * 単語を追加する。
   *
   * @param word 追加する単語。
   * @return 追加した単語のUUID。
   */
  @Nonnull
  public String addWord(UserDictWord word) {
    return rsAddWord(word);
  }

  /**
   * 単語を更新する。
   *
   * @param uuid 更新する単語のUUID。
   * @param word 新しい単語のデータ。
   */
  public void updateWord(String uuid, UserDictWord word) {
    rsUpdateWord(uuid, word);
  }

  /**
   * 単語を削除する。
   *
   * @param uuid 削除する単語のUUID。
   */
  public void removeWord(String uuid) {
    rsRemoveWord(uuid);
  }

  /**
   * ユーザー辞書をインポートする。
   *
   * @param dict インポートするユーザー辞書。
   */
  public void importDict(UserDict dict) {
    rsImportDict(dict);
  }

  /**
   * ユーザー辞書を読み込む。
   *
   * @param path ユーザー辞書のパス。
   * @throws LoadUserDictException ユーザー辞書を読み込めなかった場合。
   */
  public void load(Path path) throws LoadUserDictException {
    load(path.toString());
  }

  /**
   * ユーザー辞書を読み込む。
   *
   * @param path ユーザー辞書のパス。
   * @throws LoadUserDictException ユーザー辞書を読み込めなかった場合。
   */
  public void load(File path) throws LoadUserDictException {
    load(path.toString());
  }

  /**
   * ユーザー辞書を読み込む。
   *
   * @param path ユーザー辞書のパス。
   * @throws LoadUserDictException ユーザー辞書を読み込めなかった場合。
   */
  public void load(String path) throws LoadUserDictException {
    rsLoad(path);
  }

  /**
   * ユーザー辞書を保存する。
   *
   * @param path ユーザー辞書のパス。
   * @throws SaveUserDictException ユーザー辞書を保存できなかった場合。
   */
  public void save(Path path) throws SaveUserDictException {
    rsSave(path.toString());
  }

  /**
   * ユーザー辞書を保存する。
   *
   * @param path ユーザー辞書のパス。
   * @throws SaveUserDictException ユーザー辞書を保存できなかった場合。
   */
  public void save(File path) throws SaveUserDictException {
    rsSave(path.toString());
  }

  /**
   * ユーザー辞書を保存する。
   *
   * @param path ユーザー辞書のパス。
   * @throws SaveUserDictException ユーザー辞書を保存できなかった場合。
   */
  public void save(String path) throws SaveUserDictException {
    rsSave(path);
  }

  /**
   * ユーザー辞書の単語を取得する。
   *
   * @return ユーザー辞書の単語。
   */
  @Nonnull
  public HashMap<String, UserDictWord> toHashMap() {
    return rsToHashMap();
  }

  private native void rsNew();

  @Nonnull
  private native String rsAddWord(UserDictWord word);

  private native void rsUpdateWord(String uuid, UserDictWord word);

  private native void rsRemoveWord(String uuid);

  private native void rsImportDict(UserDict dict);

  private native void rsLoad(String path) throws LoadUserDictException;

  private native void rsSave(String path) throws SaveUserDictException;

  @Nonnull
  private native HashMap<String, UserDictWord> rsToHashMap();

  private native void rsDrop();
}
