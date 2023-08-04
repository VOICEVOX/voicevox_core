package jp.Hiroshiba.VoicevoxCore;

import java.util.HashMap;

import javax.annotation.Nonnull;
import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;

import com.google.gson.Gson;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import com.google.gson.internal.LinkedTreeMap;

/** ユーザー辞書。 */
public class UserDict extends Dll implements AutoCloseable {
  long internal;

  /** ユーザー辞書を作成する。 */
  public UserDict() {
    rsNew();
  }

  /**
   * 単語を追加する。
   *
   * @param word 追加する単語。
   * @return 追加した単語のUUID。
   */
  @Nonnull
  public String addWord(Word word) {
    Gson gson = new Gson();
    String wordJson = gson.toJson(word);

    return rsAddWord(wordJson);
  }

  /**
   * 単語を更新する。
   *
   * @param uuid 更新する単語のUUID。
   * @param word 新しい単語のデータ。
   */
  public void updateWord(String uuid, Word word) {
    Gson gson = new Gson();
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
   */
  public void load(String path) {
    rsLoad(path);
  }

  /**
   * ユーザー辞書を保存する。
   *
   * @param path ユーザー辞書のパス。
   */
  public void save(String path) {
    rsSave(path);
  }

  /**
   * ユーザー辞書の単語を取得する。
   *
   * @return ユーザー辞書の単語。
   */
  @Nonnull
  public HashMap<String, Word> getWords() {
    String json = rsGetWords();
    Gson gson = new Gson();
    @SuppressWarnings("unchecked")
    HashMap<String, LinkedTreeMap<String, ?>> rawWords = gson.fromJson(json, HashMap.class);
    if (rawWords == null) {
      throw new NullPointerException("words");
    }
    HashMap<String, Word> words = new HashMap<>();
    rawWords.forEach((uuid, rawWord) -> {
      Word word = gson.fromJson(gson.toJson(rawWord), Word.class);
      if (word == null) {
        throw new NullPointerException("word");
      }
      words.put(uuid, word);
    });

    return words;
  }

  /**
   * ユーザー辞書を破棄する。
   */
  public void close() {
    rsDrop();
  }

  private native void rsNew();

  @Nonnull
  private native String rsAddWord(String word);

  private native void rsUpdateWord(String uuid, String word);

  private native void rsRemoveWord(String uuid);

  private native void rsImportDict(UserDict dict);

  private native void rsLoad(String path);

  private native void rsSave(String path);

  @Nonnull
  private native String rsGetWords();

  private native void rsDrop();

  @Nonnull
  private static native String rsToZenkaku(String surface);

  private static native void rsValidatePronunciation(String pronunciation);

  public static class Word {
    @SerializedName("surface")
    @Expose
    @Nonnull
    public String surface;

    @SerializedName("pronunciation")
    @Expose
    @Nonnull
    public String pronunciation;

    @SerializedName("word_type")
    @Expose
    @Nonnull
    public Type wordType;

    @SerializedName("accent_type")
    @Expose
    public int accentType;

    @SerializedName("priority")
    @Expose
    @Min(0)
    @Max(10)
    public int priority;

    public Word(String surface, String pronunciation) {
      if (surface == null) {
        throw new NullPointerException("surface");
      }
      if (pronunciation == null) {
        throw new NullPointerException("pronunciation");
      }

      this.surface = rsToZenkaku(surface);
      rsValidatePronunciation(pronunciation);
      this.pronunciation = pronunciation;
      this.wordType = Type.COMMON_NOUN;
      this.accentType = 0;
      this.priority = 5;
    }

    public Word wordType(Type wordType) {
      if (wordType == null) {
        throw new NullPointerException("wordType");
      }
      this.wordType = wordType;
      return this;
    }

    public Word accentType(int accentType) {
      if (accentType < 0) {
        throw new IllegalArgumentException("accentType");
      }
      this.accentType = accentType;
      return this;
    }

    public Word priority(int priority) {
      if (priority < 0 || priority > 10) {
        throw new IllegalArgumentException("priority");
      }
      this.priority = priority;
      return this;
    }

    static enum Type {
      @SerializedName("PROPER_NOUN")
      @Expose
      PROPER_NOUN,

      @SerializedName("COMMON_NOUN")
      @Expose
      COMMON_NOUN,

      @SerializedName("VERB")
      @Expose
      VERB,

      @SerializedName("ADJECTIVE")
      @Expose
      ADJECTIVE,

      @SerializedName("SUFFIX")
      @Expose
      SUFFIX,
    }
  }
}
