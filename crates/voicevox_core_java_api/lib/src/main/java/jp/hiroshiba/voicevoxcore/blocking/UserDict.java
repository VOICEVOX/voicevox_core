package jp.hiroshiba.voicevoxcore.blocking;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.internal.LinkedTreeMap;
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
    GsonBuilder gsonBuilder = new GsonBuilder();
    gsonBuilder.registerTypeAdapter(UserDictWord.Type.class, new UserDictWord.Type.Serializer());
    Gson gson = gsonBuilder.create();
    String wordJson = gson.toJson(word);

    return rsAddWord(wordJson);
  }

  /**
   * 単語を更新する。
   *
   * @param uuid 更新する単語のUUID。
   * @param word 新しい単語のデータ。
   */
  public void updateWord(String uuid, UserDictWord word) {
    GsonBuilder gsonBuilder = new GsonBuilder();
    gsonBuilder.registerTypeAdapter(UserDictWord.Type.class, new UserDictWord.Type.Serializer());
    Gson gson = gsonBuilder.create();
    String wordJson = gson.toJson(word);

    rsUpdateWord(uuid, wordJson);
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
    String json = rsGetWords();
    GsonBuilder gsonBuilder = new GsonBuilder();
    gsonBuilder.registerTypeAdapter(UserDictWord.Type.class, new UserDictWord.Type.Deserializer());
    Gson gson = gsonBuilder.create();
    @SuppressWarnings("unchecked")
    HashMap<String, LinkedTreeMap<String, ?>> rawWords = gson.fromJson(json, HashMap.class);
    if (rawWords == null) {
      throw new NullPointerException("words");
    }
    HashMap<String, UserDictWord> words = new HashMap<>();
    rawWords.forEach(
        (uuid, rawWord) -> {
          UserDictWord word = gson.fromJson(gson.toJson(rawWord), UserDictWord.class);
          if (word == null) {
            throw new NullPointerException("word");
          }
          words.put(uuid, word);
        });

    return words;
  }

  private native void rsNew();

  @Nonnull
  private native String rsAddWord(String word);

  private native void rsUpdateWord(String uuid, String word);

  private native void rsRemoveWord(String uuid);

  private native void rsImportDict(UserDict dict);

  private native void rsLoad(String path) throws LoadUserDictException;

  private native void rsSave(String path) throws SaveUserDictException;

  @Nonnull
  private native String rsGetWords();

  private native void rsDrop();
}
