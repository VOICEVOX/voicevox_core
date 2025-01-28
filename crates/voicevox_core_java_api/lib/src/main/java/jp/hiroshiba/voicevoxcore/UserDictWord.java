package jp.hiroshiba.voicevoxcore;

import com.google.gson.JsonDeserializationContext;
import com.google.gson.JsonDeserializer;
import com.google.gson.JsonElement;
import com.google.gson.JsonParseException;
import com.google.gson.JsonPrimitive;
import com.google.gson.JsonSerializationContext;
import com.google.gson.JsonSerializer;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;

/** ユーザー辞書の単語。 */
public class UserDictWord {
  /** 単語の表層形。 */
  @SerializedName("surface")
  @Expose
  @Nonnull
  public String surface;

  /** 単語の発音。 発音として有効なカタカナである必要がある。 */
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

  /** アクセント型。 音が下がる場所を指す。 */
  @SerializedName("accent_type")
  @Expose
  public int accentType;

  /** 単語の優先度。 0から10までの整数。 数字が大きいほど優先度が高くなる。 1から9までの値を指定することを推奨。 */
  @SerializedName("priority")
  @Expose
  @Min(0)
  @Max(10)
  public int priority;

  /**
   * {@link UserDictWord}を作成する。
   *
   * @param surface 言葉の表層形。
   * @param pronunciation 言葉の発音。
   * @throws IllegalArgumentException pronunciationが不正な場合。
   */
  public UserDictWord(String surface, String pronunciation) {
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
   * @return このインスタンス。
   */
  public UserDictWord wordType(Type wordType) {
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
   * @return このインスタンス。
   */
  public UserDictWord accentType(int accentType) {
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
   * @return このインスタンス。
   * @throws IllegalArgumentException priorityが0未満または10より大きい場合。
   */
  public UserDictWord priority(int priority) {
    if (priority < 0 || priority > 10) {
      throw new IllegalArgumentException("priority");
    }
    this.priority = priority;
    return this;
  }

  @Nonnull
  private static native String rsToZenkaku(String surface);

  private static native void rsValidatePronunciation(String pronunciation);

  /** 単語の種類。 */
  public static class Type {
    /** 固有名詞。 */
    public static final Type PROPER_NOUN = new Type("PROPER_NOUN");

    /** 一般名詞。 */
    public static final Type COMMON_NOUN = new Type("COMMON_NOUN");

    /** 動詞。 */
    public static final Type VERB = new Type("VERB");

    /** 形容詞。 */
    public static final Type ADJECTIVE = new Type("ADJECTIVE");

    /** 語尾。 */
    public static final Type SUFFIX = new Type("SUFFIX");

    public static final class Serializer implements JsonSerializer<Type> {
      @Override
      public JsonElement serialize(
          Type src, java.lang.reflect.Type typeOfSrc, JsonSerializationContext context) {
        return new JsonPrimitive(src.toString());
      }
    }

    public static final class Deserializer implements JsonDeserializer<Type> {
      @Override
      public Type deserialize(
          JsonElement json, java.lang.reflect.Type typeOfT, JsonDeserializationContext context)
          throws JsonParseException {
        String value = json.getAsString();
        switch (value) {
          case "PROPER_NOUN":
            return PROPER_NOUN;
          case "COMMON_NOUN":
            return COMMON_NOUN;
          case "VERB":
            return VERB;
          case "ADJECTIVE":
            return ADJECTIVE;
          case "SUFFIX":
            return SUFFIX;
          default:
            throw new JsonParseException(String.format("Invalid value: `%s`", value));
        }
      }
    }

    private final String identifier;

    private Type(String identifier) {
      this.identifier = identifier;
    }

    @Override
    public String toString() {
      return identifier;
    }
  }
}
