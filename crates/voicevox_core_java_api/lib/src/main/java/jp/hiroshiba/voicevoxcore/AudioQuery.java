package jp.hiroshiba.voicevoxcore;

import com.google.gson.Gson;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;
import java.util.ArrayList;
import java.util.List;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidQueryException;
import jp.hiroshiba.voicevoxcore.internal.Convert;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/**
 * AudioQuery（音声合成用のクエリ）。
 *
 * <p>このクラスは不正な状態を表現しうる。どのような状態が不正なのかについては{@link #validate}を参照。このクラスを使う関数は、不正な状態に対して{@link
 * InvalidQueryException}をスローする。
 *
 * <p>コンストラクト時には、不正な状態であるかの検証は行われない。外部からのデータが不正でないことを確かめるには、コンストラクト後に{@code validate()}を用いる必要がある。
 *
 * <pre>{@code
 * AudioQuery audioQuery = (new Gson()).fromJson(json, AudioQuery.class);
 * audioQuery.validate();
 * }</pre>
 *
 * <p>GsonにおいてはVOICEVOX ENGINEに合わせる形で、フィールド名は{@link
 * #accentPhrases}のみsnake_caseとなり残りはcamelCaseとなる。ただし今後の破壊的変更にて変わる可能性がある。<a
 * href="https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md"
 * target="_blank">データのシリアライゼーション</a>を参照。
 *
 * <p>Gsonについては将来的には <a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
 * target="_blank">Jacksonに切り替わる予定</a> 。
 */
public final class AudioQuery {
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
   * 音声の総フレーム数を算出する。
   *
   * <p>音声の秒数は、フレーム数を{@link AudioFeature#FRAME_RATE}で割った値で表せる。
   *
   * <p>算出した値は32-bit環境では2<sup>32</sup>-1、64-bit環境では{@link Long#MAX_VALUE}で飽和する。算出方法は以下の通り。
   *
   * <ol>
   *   <li>以下の秒数を32-bit浮動小数点数として解釈して集める。
   *       <ul>
   *         <li>{@link #prePhonemeLength}
   *         <li>{@link #accentPhrases}の要素ごとに
   *             <ul>
   *               <li>{@link AccentPhrase#moras}の要素ごとに
   *                   <ul>
   *                     <li>{@link Mora#consonantLength}
   *                     <li>{@link Mora#vowelLength}
   *                   </ul>
   *               <li>{@link FrameLengthConfigurator#interrogativeUpspeak
   *                   interrogativeUpspeak}が{@code true}かつ{@link
   *                   AccentPhrase#isInterrogative}かつ{@link AccentPhrase#moras}の最後の{@link
   *                   Mora#pitch}が{@code 0.0}以外のとき、{@code 0.15}秒
   *               <li>{@link AccentPhrase#pauseMora}の{@link Mora#consonantLength}（通常はない）
   *               <li>{@link AccentPhrase#pauseMora}の{@link Mora#vowelLength}
   *             </ul>
   *         <li>{@link #postPhonemeLength}
   *       </ul>
   *   <li>それぞれの秒数を{@code secs}として、対応するフレーム長を<code>roundTiesEven(roundTiesEven(secs * {@link
   *       AudioFeature#FRAME_RATE}) / {@link #speedScale})</code>として算出する。ここで{@code
   *       roundTiesEven}は<a
   *       href="https://doc.rust-lang.org/stable/std/primitive.f32.html#method.round_ties_even">Rustの{@code
   *       f32::round_ties_even}</a>であり、{@link Math#rint(double) Math.rint}と同様IEEE 754の{@code
   *       roundToIntegralTiesToEven}演算を行う。{@link
   *       #speedScale}も32-bit浮動小数点数として解釈し、乗算と除算も32-bit浮動小数点数上で行う。
   *   <li>各フレーム長を足し合わせる。
   * </ol>
   *
   * <p>{@code AudioQuery}に対応する音声の長さは将来的に変わる可能性がある。例えば、秒数を64-bit浮動小数点数として解釈しているVOICEVOX
   * ENGINEと挙動を揃える可能性がある。
   *
   * <p>使用例:
   *
   * <pre>{@code
   * AudioQuery query = synth.createAudioQuery("こんにちは、音声合成の世界へようこそ", WHATEVER_STYLE1);
   * AudioFeature audio = synth.createAudioFeature(query, WHATEVER_STYLE2).perform();
   *
   * assert query.frameLength().calculate() == audio.getFrameLength();
   * }</pre>
   *
   * <pre>{@code
   * void main() {
   *   AudioQuery query = AudioQuery.fromAccentPhrases(Arrays.asList());
   *   query.speedScale = 1.2;
   *   query.prePhonemeLength = 3.3;
   *   query.postPhonemeLength = 4.4;
   *
   *   assert query.frameLength().calculate()
   *       // `speed_scale`, `pre_phoneme_length`
   *       == toFrameLength((float) query.prePhonemeLength, (float) query.speedScale)
   *           // `speed_scale`, `consonant_length`, `vowel_length`, `is_interrogative`
   *           + 0
   *           // `speed_scale`, `post_phoneme_length`
   *           + toFrameLength((float) query.postPhonemeLength, (float) query.speedScale);
   * }
   *
   * private static long toFrameLength(float secs, float speedScale) {
   *   final float FRAME_RATE = (float) AudioFeature.FRAME_RATE;
   *   return (long) roundevenf(roundevenf(secs * FRAME_RATE) / speedScale);
   * }
   *
   * private static float roundevenf(float value) {
   *   return (float) Math.rint(value);
   * }
   * }</pre>
   *
   * <pre>{@code
   * AudioQuery query = AudioQuery.fromAccentPhrases(Arrays.asList());
   * query.speedScale = Float.MIN_NORMAL;
   * assert query.frameLength().calculate() == Long.MAX_VALUE; // 64-bit環境の場合
   * }</pre>
   *
   * @return {@link FrameLengthConfigurator}。
   * @see FrameLengthConfigurator#calculate
   */
  @Nonnull
  public FrameLengthConfigurator frameLength() {
    return new FrameLengthConfigurator(this);
  }

  /**
   * このインスタンスが不正であるときエラーを返す。
   *
   * <p>不正であるとは、{@code @throws}で示す条件を満たすことである。
   *
   * <p>また次の状態に対してはログで警告を出す。将来的にはエラーになる予定。
   *
   * <ul>
   *   <li>{@link #outputSamplingRate}が{@code 24000}以外の値（エラーと同様将来的に解消予定）。
   * </ul>
   *
   * @throws InvalidQueryException 次のうちどれかを満たす場合
   *     <ul>
   *       <li>JSONへのシリアライズが不可。
   *           <ul>
   *             <li>{@link #speedScale}がNaNもしくは±infinity。
   *             <li>{@link #pitchScale}がNaNもしくは±infinity。
   *             <li>{@link #intonationScale}がNaNもしくは±infinity。
   *             <li>{@link #volumeScale}がNaNもしくは±infinity。
   *             <li>{@link #prePhonemeLength}がNaNもしくは±infinity。
   *             <li>{@link #postPhonemeLength}がNaNもしくは±infinity。
   *           </ul>
   *       <li><a
   *           href="https://voicevox.github.io/voicevox_core/apis/rust_api/voicevox_core/struct.AudioQuery.html">Rust
   *           APIの{@code AudioQuery}型</a>としてデシリアライズ不可。
   *           <ul>
   *             <li>{@link #speedScale}が負。
   *             <li>{@link #volumeScale}が負。
   *             <li>{@link #prePhonemeLength}が負。
   *             <li>{@link #postPhonemeLength}が負。
   *             <li>{@link #outputSamplingRate}が以下の値を取る。
   *                 <ul>
   *                   <li>{@code 0}以下の値。
   *                   <li>{@code 24000}の倍数以外（将来的に解消予定。cf. <a
   *                       href="https://github.com/VOICEVOX/voicevox_core/issues/762">#762</a>）。
   *                 </ul>
   *           </ul>
   *       <li>{@link #accentPhrases}の要素のうちいずれかが不正。
   *     </ul>
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
    String queryJson =
        rsFromAccentPhrases(Convert.jsonFromQueryLike(accentPhrases, "不正なアクセント句の列です"));
    AudioQuery query = gson.fromJson(queryJson, AudioQuery.class);
    if (query == null) {
      throw new NullPointerException();
    }
    return query;
  }

  @Nonnull
  private static native String rsFromAccentPhrases(String accentPhrases);

  private native long rsFrameLength(boolean enableInterrogativeUpspeak);

  private native void rsValidate();

  /** {@link AudioQuery#frameLength} のオプション。 */
  public final class FrameLengthConfigurator {
    private AudioQuery audioQuery;
    private boolean interrogativeUpspeak; // FIXME: デフォルトで`false`になってしまっている！

    private FrameLengthConfigurator(AudioQuery audioQuery) {
      this.audioQuery = audioQuery;
      this.interrogativeUpspeak = false;
    }

    /**
     * 疑問文の調整を有効にするかどうか。
     *
     * @param interrogativeUpspeak 疑問文の調整を有効にするかどうか。
     * @return {@link FrameLengthConfigurator}。
     */
    @Nonnull
    public FrameLengthConfigurator interrogativeUpspeak(boolean interrogativeUpspeak) {
      this.interrogativeUpspeak = interrogativeUpspeak;
      return this;
    }

    /**
     * 音声の総フレーム数を算出する。
     *
     * @return 総フレーム数
     * @throws InvalidQueryException {@link AudioQuery}が<a
     *     href="https://voicevox.github.io/voicevox_core/apis/rust_api/voicevox_core/struct.AudioQuery.html">Rust
     *     APIの{@code AudioQuery}型</a>としてデシリアライズ不可の場合
     * @see AudioQuery#frameLength
     */
    public long calculate() {
      return audioQuery.rsFrameLength(interrogativeUpspeak);
    }
  }
}
