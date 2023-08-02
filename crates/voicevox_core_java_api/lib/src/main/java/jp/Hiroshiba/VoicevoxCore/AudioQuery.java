package jp.Hiroshiba.VoicevoxCore;

import java.util.ArrayList;
import java.util.List;

import javax.annotation.Nonnull;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;

public class AudioQuery {
  @JsonProperty("accent_phrases")
  @SerializedName("accent_phrases")
  @Expose
  @Nonnull
  public List<AccentPhrase> accentPhrases;

  @JsonProperty("speed_scale")
  @SerializedName("speed_scale")
  @Expose
  public double speedScale;

  @JsonProperty("pitch_scale")
  @SerializedName("pitch_scale")
  @Expose
  public double pitchScale;

  @JsonProperty("intonation_scale")
  @SerializedName("intonation_scale")
  @Expose
  public double intonationScale;

  @JsonProperty("volume_scale")
  @SerializedName("volume_scale")
  @Expose
  public double volumeScale;

  @JsonProperty("pre_phoneme_length")
  @SerializedName("pre_phoneme_length")
  @Expose
  public double prePhonemeLength;

  @JsonProperty("post_phoneme_length")
  @SerializedName("post_phoneme_length")
  @Expose
  public double postPhonemeLength;

  @JsonProperty("output_sampling_rate")
  @SerializedName("output_sampling_rate")
  @Expose
  public int outputSamplingRate;

  @JsonProperty("output_stereo")
  @SerializedName("output_stereo")
  @Expose
  public boolean outputStereo;

  @JsonProperty("kana")
  @SerializedName("kana")
  @Expose
  @Nonnull
  public String kana;

  public AudioQuery() {
    this.accentPhrases = new ArrayList<>();
    this.speedScale = 1.0;
    this.pitchScale = 0.0;
    this.intonationScale = 1.0;
    this.volumeScale = 1.0;
    this.prePhonemeLength = 0.1;
    this.postPhonemeLength = 0.1;
    this.outputSamplingRate = 24000;
    this.kana = "";
  }
}
