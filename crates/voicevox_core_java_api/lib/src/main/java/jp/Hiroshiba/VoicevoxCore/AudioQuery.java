package jp.Hiroshiba.VoicevoxCore;

import java.util.ArrayList;
import java.util.List;

import javax.annotation.Nonnull;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;

/** AudioQuery (音声合成用のクエリ)。 */
public class AudioQuery {
  /** アクセント句の配列。 */
  @JsonProperty("accent_phrases")
  @SerializedName("accent_phrases")
  @Expose
  @Nonnull
  public List<AccentPhrase> accentPhrases;

  /** 全体の話速。 */
  @JsonProperty("speed_scale")
  @SerializedName("speed_scale")
  @Expose
  public double speedScale;

  /** 全体の音高。 */
  @JsonProperty("pitch_scale")
  @SerializedName("pitch_scale")
  @Expose
  public double pitchScale;

  /** 全体の抑揚。 */
  @JsonProperty("intonation_scale")
  @SerializedName("intonation_scale")
  @Expose
  public double intonationScale;

  /** 全体の音量。 */
  @JsonProperty("volume_scale")
  @SerializedName("volume_scale")
  @Expose
  public double volumeScale;

  /** 音声の前の無音時間。 */
  @JsonProperty("pre_phoneme_length")
  @SerializedName("pre_phoneme_length")
  @Expose
  public double prePhonemeLength;

  /** 音声の後の無音時間。 */
  @JsonProperty("post_phoneme_length")
  @SerializedName("post_phoneme_length")
  @Expose
  public double postPhonemeLength;

  /** 音声データの出力サンプリングレート。 */
  @JsonProperty("output_sampling_rate")
  @SerializedName("output_sampling_rate")
  @Expose
  public int outputSamplingRate;

  /** 音声データをステレオ出力するか否か。 */
  @JsonProperty("output_stereo")
  @SerializedName("output_stereo")
  @Expose
  public boolean outputStereo;

  /**
   * [読み取り専用] AquesTalk風記法。
   *
   * {@link Synthesizer#audioQuery} が返すもののみ ``str`` となる。入力としてのAudioQueryでは無視される。
   */
  @JsonProperty("kana")
  @SerializedName("kana")
  @Expose
  @Nonnull
  final public String kana;

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
