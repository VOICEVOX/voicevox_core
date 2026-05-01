package jp.hiroshiba.voicevoxcore;

import jakarta.annotation.Nonnull;
import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;

/**
 * ユーザー辞書の単語。
 *
 * <p>{@code UserDict.load}および{@code UserDict.save}におけるJSONの形式はVOICEVOX
 * ENGINEに合わせた形となっており、ビルダーおよびゲッターで扱う構造とは大幅に異なる。ただし今後の破壊的変更にて変わる可能性がある。<a
 * href="https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md"
 * target="_blank">データのシリアライゼーション</a>を参照。
 *
 * <p>現在この型はGsonをサポートしていない。将来的には<a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
 * target="_blank">Jacksonに対応する予定</a> 。
 */
public class UserDictWord {
  /** 単語の表層形。 */
  @Nonnull public String surface;

  /** 単語の発音。 発音として有効なカタカナである必要がある。 */
  @Nonnull public String pronunciation;

  /**
   * 単語の種類。
   *
   * @see Type
   */
  @Nonnull public Type wordType;

  /** アクセント型。 音が下がる場所を指す。 */
  public int accentType;

  /** 単語の優先度。 0から10までの整数。 数字が大きいほど優先度が高くなる。 1から9までの値を指定することを推奨。 */
  @Min(0)
  @Max(10)
  public int priority;

  /**
   * {@link UserDictWord}を作成する。
   *
   * @param surface 言葉の表層形。
   * @param pronunciation 言葉の発音。
   * @param accentType アクセント型。
   * @throws IllegalArgumentException pronunciationが不正な場合。
   */
  public UserDictWord(String surface, String pronunciation, int accentType) {
    if (surface == null) {
      throw new NullPointerException("surface");
    }
    if (pronunciation == null) {
      throw new NullPointerException("pronunciation");
    }
    if (accentType < 0) {
      throw new IllegalArgumentException("accentType");
    }

    this.surface = rsToZenkaku(surface);
    rsValidatePronunciation(pronunciation);
    this.pronunciation = pronunciation;
    this.wordType = Type.COMMON_NOUN;
    this.accentType = accentType;
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
