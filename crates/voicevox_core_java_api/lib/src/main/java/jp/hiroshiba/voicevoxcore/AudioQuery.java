package jp.hiroshiba.voicevoxcore;

import com.google.gson.Gson;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;
import java.util.ArrayList;
import java.util.List;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidQueryException;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/**
 * AudioQuery（音声合成用のクエリ）。
 *
 * <p>この構造体の状態によっては、{@code Synthesizer}の各メソッドは{@link InvalidQueryException}を発する。詳細は{@link
 * #validate}にて。
 *
 * <p>GsonにおいてはVOICEVOX ENGINEに合わせる形で、フィールド名は{@link
 * #accentPhrases}のみsnake_caseとなり残りはcamelCaseとなる。ただし今後の破壊的変更にて変わる可能性がある。<a
 * href="https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md"
 * target="_blank">データのシリアライゼーション</a>を参照。
 *
 * <p>Gsonについては将来的には <a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
 * target="_blank">Jacksonに切り替わる予定</a> 。
 */
public class AudioQuery {
  static {
    Dll.loadLibrary();
  }

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
    this.kana = null;
  }

  /**
   * このインスタンスをバリデートする。
   *
   * <p>次のうちどれかを満たすなら{@link InvalidQueryException}を発する。
   *
   * <ul>
   *   <li>{@link #accentPhrases}の要素のうちいずれかが不正。
   *   <li>{@link #outputSamplingRate}が{@code 24000}の倍数ではない、もしくは{@code 0} (将来的に解消予定。cf. <a
   *       href="https://github.com/VOICEVOX/voicevox_core/issues/762">#762</a>)
   * </ul>
   *
   * <p>また次の状態に対してはログで警告を出す。将来的にはエラーになる予定。
   *
   * <ul>
   *   <li>{@link #accentPhrases}の要素のうちいずれかが警告が出る状態。
   *   <li>{@link #speedScale}がNaN、infinity、もしくは負。
   *   <li>{@link #pitchScale}がNaNもしくは±infinity。
   *   <li>{@link #intonationScale}がNaNもしくは±infinity。
   *   <li>{@link #volumeScale}がNaN、infinity、もしくは負。
   *   <li>{@link #prePhonemeLength}がNaN、infinity、もしくは負。
   *   <li>{@link #postPhonemeLength}がNaN、infinity、もしくは負。
   *   <li>{@link #outputSamplingRate}が{@code 24000}以外の値（エラーと同様将来的に解消予定）。
   * </ul>
   */
  public void validate() {
    rsValidate();
  }

  @Override
  public boolean equals(Object obj) {
    if (!(obj instanceof AudioQuery)) {
      return false;
    }
    AudioQuery other = (AudioQuery) obj;
    return accentPhrases.equals(other.accentPhrases)
        && speedScale == other.speedScale
        && pitchScale == other.pitchScale
        && intonationScale == other.intonationScale
        && volumeScale == other.volumeScale
        && prePhonemeLength == other.prePhonemeLength
        && postPhonemeLength == other.postPhonemeLength
        && outputSamplingRate == other.outputSamplingRate
        && outputStereo == other.outputStereo;
  }

  public static AudioQuery fromAccentPhrases(List<AccentPhrase> accentPhrases) {
    Gson gson = new Gson();
    String queryJson = rsFromAccentPhrases(gson.toJson(accentPhrases));
    AudioQuery query = gson.fromJson(queryJson, AudioQuery.class);
    if (query == null) {
      throw new NullPointerException();
    }
    return query;
  }

  @Nonnull
  private static native String rsFromAccentPhrases(String accentPhrases);

  private native void rsValidate();
}
