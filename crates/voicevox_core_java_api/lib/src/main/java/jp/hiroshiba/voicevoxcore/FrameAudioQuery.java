package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import jakarta.annotation.Nonnull;
import java.util.ArrayList;
import java.util.List;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidQueryException;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/**
 * フレームごとの音声合成用のクエリ。
 *
 * <p>このクラスは不正な状態を表現しうる。どのような状態が不正なのかについては{@link #validate}を参照。このクラスを使う関数は、不正な状態に対して{@link
 * InvalidQueryException}をスローする。
 *
 * <p>コンストラクト時には、不正な状態であるかの検証は行われない。外部からのデータが不正でないことを確かめるには、コンストラクト後に{@code validate()}を用いる必要がある。
 *
 * <pre>{@code
 * FrameAudioQuery frameAudioquery = (new Gson()).fromJson(json, FrameAudioQuery.class);
 * frameAudioquery.validate();
 * }</pre>
 *
 * <p>現在この型はGSONに対応しているが、将来的には <a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
 * target="_blank">Jacksonに切り替わる予定</a> 。
 */
public final class FrameAudioQuery {
  static {
    Dll.loadLibrary();
  }

  /** フレームごとの基本周波数。 */
  @Expose @Nonnull public float[] f0;

  /** フレームごとの音量。 */
  @Expose @Nonnull public float[] volume;

  /** 音素のリスト。 */
  @Expose @Nonnull public List<FramePhoneme> phonemes;

  /** 全体の音量。 */
  @Expose public float volumeScale;

  /** 音声データの出力サンプリングレート。 */
  @Expose public int outputSamplingRate;

  /** 音声データをステレオ出力するか否か。 */
  @Expose public boolean outputStereo;

  private FrameAudioQuery() {
    f0 = new float[] {};
    volume = new float[] {};
    phonemes = new ArrayList<>();
    volumeScale = 1f;
    outputSamplingRate = 24000;
    outputStereo = false;
  }

  public FrameAudioQuery(
      @Nonnull float[] f0,
      @Nonnull float[] volume,
      @Nonnull List<FramePhoneme> phonemes,
      float volumeScale,
      int outputSamplingRate,
      boolean outputStereo) {
    this.f0 = f0;
    this.volume = volume;
    this.phonemes = phonemes;
    this.volumeScale = volumeScale;
    this.outputSamplingRate = outputSamplingRate;
    this.outputStereo = outputStereo;
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
   *             <li>{@link #f0}の要素がNaNもしくは±infinity。
   *             <li>{@link #volume}の要素がNaNもしくは±infinity。
   *             <li>{@link #volumeScale}がNaNもしくは±infinity。
   *           </ul>
   *       <li><a
   *           href="https://voicevox.github.io/voicevox_core/apis/rust_api/voicevox_core/struct.FrameAudioQuery.html">Rust
   *           APIの{@code FrameAudioQuery}型</a>としてデシリアライズ不可。
   *           <ul>
   *             <li>{@link #f0}の要素が負。
   *             <li>{@link #phonemes}の要素のうちいずれかが不正。
   *             <li>{@link #volumeScale}が負。
   *             <li>{@link #outputSamplingRate}が以下の値を取る。
   *                 <ul>
   *                   <li>{@code 0}以下の値。
   *                   <li>2<sup>32</sup>-1を超過する値。
   *                   <li>{@code 24000}の倍数以外（将来的に解消予定。cf. <a
   *                       href="https://github.com/VOICEVOX/voicevox_core/issues/762">#762</a>）。
   *                 </ul>
   *           </ul>
   *     </ul>
   */
  public void validate() {
    rsValidate();
  }

  // `phonemes`の型が`List`のため、`clone`は実装できない

  private native void rsValidate();
}
