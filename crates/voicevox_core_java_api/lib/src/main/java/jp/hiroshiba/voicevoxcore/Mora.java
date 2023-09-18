package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;

/** モーラ（子音＋母音）ごとの情報。 */
public class Mora {
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
}
