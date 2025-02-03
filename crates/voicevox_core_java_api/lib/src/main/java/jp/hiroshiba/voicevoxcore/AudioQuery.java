package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;
import java.util.ArrayList;
import java.util.List;

/** AudioQuery（音声合成用のクエリ）。 */
public class AudioQuery {
  /** アクセント句の配列。 */
  @SerializedName("accent_phrases")
  @Expose
  @Nonnull
  public List<AccentPhrase> accentPhrases;

  /** 全体の話速。 */
  @Expose public double speedScale;

  /** 全体の音高。 */
  @Expose public double pitchScale;

  /** 全体の抑揚。 */
  @Expose public double intonationScale;

  /** 全体の音量。 */
  @Expose public double volumeScale;

  /** 音声の前の無音時間。 */
  @Expose public double prePhonemeLength;

  /** 音声の後の無音時間。 */
  @Expose public double postPhonemeLength;

  /** 音声データの出力サンプリングレート。 */
  @Expose public int outputSamplingRate;

  /** 音声データをステレオ出力するか否か。 */
  @Expose public boolean outputStereo;

  /** 句読点などの無音時間。{@code null}のときは無視される。デフォルト値は{@code null}。 */
  @Expose @Nullable public Double pauseLength;

  /** 読点などの無音時間（倍率）。デフォルト値は{@code 1.}。 */
  @Expose public double pauseLengthScale;

  /**
   * [読み取り専用] AquesTalk風記法。
   *
   * <p>{@link jp.hiroshiba.voicevoxcore.blocking.Synthesizer#createAudioQuery} が返すもののみ String
   * となる。入力としてのAudioQueryでは無視される。
   */
  @Expose @Nullable public final String kana;

  public AudioQuery() {
    this.accentPhrases = new ArrayList<>();
    this.speedScale = 1.0;
    this.pitchScale = 0.0;
    this.intonationScale = 1.0;
    this.volumeScale = 1.0;
    this.prePhonemeLength = 0.1;
    this.postPhonemeLength = 0.1;
    this.outputSamplingRate = 24000;
    this.pauseLength = null;
    this.pauseLengthScale = 1.0;
    this.kana = null;
  }
}
