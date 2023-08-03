package jp.Hiroshiba.VoicevoxCore;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;

/** モーラ（子音＋母音）ごとの情報。 */
public class Mora {
  /** 文字。 */
  @JsonProperty("text")
  @SerializedName("text")
  @Expose
  @Nonnull
  @SuppressWarnings("NullableProblems")
  public String text;

  /** 子音の音素。 */
  @JsonProperty("consonant")
  @SerializedName("consonant")
  @Expose
  @Nullable
  public String consonant;

  /** 子音の音長。 */
  @JsonProperty("consonant_length")
  @SerializedName("consonant_length")
  @Expose
  public java.lang.Double consonantLength;

  /** 母音の音素。 */
  @JsonProperty("vowel")
  @SerializedName("vowel")
  @Expose
  @Nonnull
  @SuppressWarnings("NullableProblems")
  public String vowel;

  /** 母音の音長。 */
  @JsonProperty("vowel_length")
  @SerializedName("vowel_length")
  @Expose
  public double vowelLength;

  /** 音高。 */
  @JsonProperty("pitch")
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
