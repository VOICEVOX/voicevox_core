package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import java.util.ArrayList;
import java.util.List;
import javax.annotation.Nonnull;
import javax.annotation.Nullable;

/** AudioQuery（音声合成用のクエリ）。 */
public class AudioQuery {
  /** アクセント句の配列。 */
  @SerializedName("accent_phrases")
  @Expose
  @Nonnull
  public List<AccentPhrase> accentPhrases;

  /** 全体の話速。 */
  @SerializedName("speed_scale")
  @Expose
  public double speedScale;

  /** 全体の音高。 */
  @SerializedName("pitch_scale")
  @Expose
  public double pitchScale;

  /** 全体の抑揚。 */
  @SerializedName("intonation_scale")
  @Expose
  public double intonationScale;

  /** 全体の音量。 */
  @SerializedName("volume_scale")
  @Expose
  public double volumeScale;

  /** 音声の前の無音時間。 */
  @SerializedName("pre_phoneme_length")
  @Expose
  public double prePhonemeLength;

  /** 音声の後の無音時間。 */
  @SerializedName("post_phoneme_length")
  @Expose
  public double postPhonemeLength;

  /** 音声データの出力サンプリングレート。 */
  @SerializedName("output_sampling_rate")
  @Expose
  public int outputSamplingRate;

  /** 音声データをステレオ出力するか否か。 */
  @SerializedName("output_stereo")
  @Expose
  public boolean outputStereo;

  /**
   * [読み取り専用] AquesTalk風記法。
   *
   * <p>{@link Synthesizer#createAudioQuery} が返すもののみ String となる。入力としてのAudioQueryでは無視される。
   */
  @SerializedName("kana")
  @Expose
  @Nullable
  public final String kana;

  public AudioQuery() {
    this.accentPhrases = new ArrayList<>();
    this.speedScale = 1.0;
    this.pitchScale = 0.0;
    this.intonationScale = 1.0;
    this.volumeScale = 1.0;
    this.prePhonemeLength = 0.1;
    this.postPhonemeLength = 0.1;
    this.outputSamplingRate = 24000;
    this.kana = null;
  }
}
