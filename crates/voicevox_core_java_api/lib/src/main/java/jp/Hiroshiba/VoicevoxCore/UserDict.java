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

  /**
   * ユーザー辞書の単語。
   */
  public static class Word {
    /**
     * 単語の表層形。
     */
    @SerializedName("surface")
    @Expose
    @Nonnull
    public String surface;

    /**
     * 単語の発音。
     * 発音として有効なカタカナである必要がある。
     */
    @SerializedName("pronunciation")
    @Expose
    @Nonnull
    public String pronunciation;

    /**
     * 単語の種類。
     *
     * @see Type
     */
    @SerializedName("word_type")
    @Expose
    @Nonnull
    public Type wordType;

    /**
     * アクセント型。
     * 音が下がる場所を指す。
     */
    @SerializedName("accent_type")
    @Expose
    public int accentType;

    /**
     * 単語の優先度。
     * 0から10までの整数。
     * 数字が大きいほど優先度が高くなる。
     * 1から9までの値を指定することを推奨。
     */
    @SerializedName("priority")
    @Expose
    @Min(0)
    @Max(10)
    public int priority;

    /**
     * UserDict.Wordを作成する。
     *
     * @param surface       言葉の表層形。
     * @param pronunciation 言葉の発音。
     *
     * @throws IllegalArgumentException pronunciationが不正な場合。
     */
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

    /**
     * 単語の種類を設定する。
     *
     * @param wordType 単語の種類。
     *
     * @return このインスタンス。
     */
    public Word wordType(Type wordType) {
      if (wordType == null) {
        throw new NullPointerException("wordType");
      }
      this.wordType = wordType;
      return this;
    }

    /**
     * アクセント型を設定する。
     *
     * @param accentType アクセント型。
     *
     * @return このインスタンス。
     */
    public Word accentType(int accentType) {
      if (accentType < 0) {
        throw new IllegalArgumentException("accentType");
      }
      this.accentType = accentType;
      return this;
    }

    /**
     * 優先度を設定する。
     *
     * @param priority 優先度。
     *
     * @return このインスタンス。
     *
     * @throws IllegalArgumentException priorityが0未満または10より大きい場合。
     */
    public Word priority(int priority) {
      if (priority < 0 || priority > 10) {
        throw new IllegalArgumentException("priority");
      }
      this.priority = priority;
      return this;
    }

    /**
     * 単語の種類。
     */
    public static enum Type {
      /**
       * 固有名詞。
       */
      @SerializedName("PROPER_NOUN")
      @Expose
      PROPER_NOUN,

      /**
       * 一般名詞。
       */
      @SerializedName("COMMON_NOUN")
      @Expose
      COMMON_NOUN,

      /**
       * 動詞。
       */
      @SerializedName("VERB")
      @Expose
      VERB,

      /**
       * 形容詞。
       */
      @SerializedName("ADJECTIVE")
      @Expose
      ADJECTIVE,

      /**
       * 語尾。
       */
      @SerializedName("SUFFIX")
      @Expose
      SUFFIX,
    }
  }
}
