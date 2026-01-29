package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import jakarta.annotation.Nonnull;
import java.util.ArrayList;
import java.util.List;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidQueryException;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/**
 * 楽譜情報。
 *
 * <pre>{@code
 * Score score =
 *     (new Gson())
 *         .fromJson(
 *             "{"
 *                 + "  \"notes\": [ "
 *                 + "    { \"key\": null, \"frame_length\": 15, \"lyric\": \"\" },"
 *                 + "    { \"key\": 60, \"frame_length\": 45, \"lyric\": \"ド\" },"
 *                 + "    { \"key\": 62, \"frame_length\": 45, \"lyric\": \"レ\" },"
 *                 + "    { \"key\": 64, \"frame_length\": 45, \"lyric\": \"ミ\" },"
 *                 + "    { \"key\": null, \"frame_length\": 15, \"lyric\": \"\" }"
 *                 + "  ]"
 *                 + "}",
 *             Score.class);
 * }</pre>
 *
 * <p>このクラスは不正な状態を表現しうる。どのような状態が不正なのかについては{@link #validate}を参照。このクラスを使う関数は、不正な状態に対して{@link
 * InvalidQueryException}をスローする。
 *
 * <p>コンストラクト時には、不正な状態であるかの検証は行われない。外部からのデータが不正でないことを確かめるには、コンストラクト後に{@code validate()}を用いる必要がある。
 *
 * <pre>{@code
 * Score score = (new Gson()).fromJson(json, Score.class);
 * score.validate();
 * }</pre>
 *
 * <p>現在この型はGSONに対応しているが、将来的には <a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
 * target="_blank">Jacksonに切り替わる予定</a> 。
 */
public final class Score {
  static {
    Dll.loadLibrary();
  }

  /** 音符のリスト。 */
  @Expose @Nonnull public List<Note> notes;

  private Score() {
    notes = new ArrayList<>();
  }

  public Score(@Nonnull List<Note> notes) {
    this.notes = notes;
  }

  /**
   * このインスタンスが不正であるときエラーを返す。
   *
   * <p>不正であるとは、{@code @throws}で示す条件を満たすことである。
   *
   * @throws InvalidQueryException 次のうちどれかを満たす場合
   *     <ul>
   *       <li>{@link #notes}の要素のうちいずれかが不正。
   *       <li>{@link #notes}が空であるか、もしくは先頭が音符。
   *     </ul>
   */
  public void validate() {
    rsValidate();
  }

  // `notes`の型が`List`のため、`clone`は実装できない

  private native void rsValidate();
}
