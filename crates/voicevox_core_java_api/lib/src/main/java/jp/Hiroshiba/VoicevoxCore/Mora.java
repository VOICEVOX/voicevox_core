package jp.Hiroshiba.VoicevoxCore;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;

public class Mora {
  @JsonProperty("text")
  @SerializedName("text")
  @Expose
  @Nonnull
  @SuppressWarnings("NullableProblems")
  public String text;

  @JsonProperty("consonant")
  @SerializedName("consonant")
  @Expose
  @Nullable
  public String consonant;

  @JsonProperty("consonant_length")
  @SerializedName("consonant_length")
  @Expose
  public java.lang.Double consonantLength;

  @JsonProperty("vowel")
  @SerializedName("vowel")
  @Expose
  @Nonnull
  @SuppressWarnings("NullableProblems")
  public String vowel;

  @JsonProperty("vowel_length")
  @SerializedName("vowel_length")
  @Expose
  public double vowelLength;

  @JsonProperty("pitch")
  @SerializedName("pitch")
  @Expose
  public double pitch;

  public Mora() {
    this.text = "";
    this.consonant = "";
    this.consonantLength = 0.0;
    this.vowel = "";
    this.vowelLength = 0.0;
    this.pitch = 0.0;
  }
}
