package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidQueryException;

/**
 * モーラ（子音＋母音）ごとの情報。
 *
 * <p>この構造体の状態によっては、{@code Synthesizer}の各メソッドは{@link InvalidQueryException}を発する。詳細は{@link
 * #validate}にて。
 *
 * <p>Gsonにおいてはフィールド名はsnake_caseとなる。<a
 * href="https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md"
 * target="_blank">データのシリアライゼーション</a>を参照。
 *
 * <p>Gsonについては将来的には <a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
 * target="_blank">Jacksonに切り替わる予定</a> 。
 */
public class Mora implements Cloneable {
  /** 文字。 */
  @SerializedName("text")
  @Expose
  @Nonnull
  @SuppressWarnings("NullableProblems")
  public String text;

  /** 子音の音素。 */
  @SerializedName("consonant")
  @Expose
  @Nullable
  public String consonant;

  /** 子音の音長。 */
  @SerializedName("consonant_length")
  @Expose
  public java.lang.Double consonantLength;

  /** 母音の音素。 */
  @SerializedName("vowel")
  @Expose
  @Nonnull
  @SuppressWarnings("NullableProblems")
  public String vowel;

  /** 母音の音長。 */
  @SerializedName("vowel_length")
  @Expose
  public double vowelLength;

  /** 音高。 */
  @SerializedName("pitch")
  @Expose
  public double pitch;

  public Mora() {
    this.text = "";
    this.consonant = null;
    this.consonantLength = null;
    this.vowel = "";
    this.vowelLength = 0.0;
    this.pitch = 0.0;
  }

  /**
   * このインスタンスをバリデートする。
   *
   * <p>次のうちどれかを満たすなら{@link InvalidQueryException}を発する。
   *
   * <ul>
   *   <li>JSONへのシリアライズが不可。
   *       <ul>
   *         <li>{@link #consonantLength}がNaN、infinity、もしくは負。
   *         <li>{@link #vowelLength}がNaN、infinity、もしくは負。
   *         <li>{@link #pitch}がNaNもしくは±infinity。
   *       </ul>
   *   <li>{@link #consonant}と{@link #consonantLength}の有無が不一致。
   *   <li>{@link #consonant}が子音以外の音素であるか、もしくは音素として不正。
   *   <li>{@link #vowel}が子音であるか、もしくは音素として不正。
   * </ul>
   *
   * <p>また次の状態に対してはログで警告を出す。将来的にはエラーになる予定。
   *
   * <ul>
   *   <li>{@link #consonantLength}が負。
   *   <li>{@link #vowelLength}が負。
   * </ul>
   */
  public void validate() {
    rsValidate();
  }

  @Override
  public boolean equals(Object obj) {
    if (!(obj instanceof Mora)) {
      return false;
    }
    Mora other = (Mora) obj;
    return text.equals(other.text)
        && consonant.equals(other.consonant)
        && vowel.equals(other.vowel)
        && vowelLength == other.vowelLength
        && pitch == other.pitch;
  }

  @Override
  public Mora clone() {
    Mora ret = new Mora();
    ret.text = text;
    ret.consonant = consonant;
    ret.consonantLength = consonantLength;
    ret.vowel = vowel;
    ret.vowelLength = vowelLength;
    ret.pitch = pitch;
    return ret;
  }

  private native void rsValidate();
}
