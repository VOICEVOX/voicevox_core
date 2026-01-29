package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidQueryException;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/**
 * 音符または休符。
 *
 * <pre>{@code
 * Note note =
 *     (new Gson())
 *         .fromJson("{ \"key\": 66, \"frame_length\": 45, \"lyric\": \"ファ\" }", Note.class);
 *
 * Note rest =
 *     (new Gson())
 *         .fromJson("{ \"key\": null, \"frame_length\": 45, \"lyric\": \"\" }", Note.class);
 * }</pre>
 *
 * <p>このクラスは不正な状態を表現しうる。どのような状態が不正なのかについては{@link #validate}を参照。このクラスを使う関数は、不正な状態に対して{@link
 * InvalidQueryException}をスローする。
 *
 * <p>コンストラクト時には、不正な状態であるかの検証は行われない。外部からのデータが不正でないことを確かめるには、コンストラクト後に{@code validate()}を用いる必要がある。
 *
 * <pre>{@code
 * Note note = (new Gson()).fromJson(json, Note.class);
 * note.validate();
 * }</pre>
 *
 * <p>Gsonにおいてはフィールド名はsnake_caseとなる。<a
 * href="https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md"
 * target="_blank">データのシリアライゼーション</a>を参照。
 *
 * <p>Gsonについては将来的には <a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
 * target="_blank">Jacksonに切り替わる予定</a> 。
 */
public final class Note implements Cloneable {
  static {
    Dll.loadLibrary();
  }

  /**
   * ID。
   *
   * <p>{@link FrameAudioQuery}を生成するときに{@link FramePhoneme#noteId}にコピーされる。歌唱音声には影響しない。
   */
  @Expose @Nullable public String id;

  /**
   * 音階。
   *
   * <ul>
   *   <li>音符の場合、MIDIのnote number（例: C4なら{@code 60}）。
   *   <li>休符の場合、{@code null}。
   * </ul>
   */
  @Expose @Nullable public Byte key;

  /**
   * 歌詞。
   *
   * <ul>
   *   <li>音符の場合、一つのモーラを表すひらがな/カタカナ（例: {@code "ド"}, {@code "ファ"}）。
   *   <li>休符の場合、空文字列。
   * </ul>
   */
  @Expose @Nonnull public String lyric;

  /**
   * 音符のフレーム長。
   *
   * <p>秒数に93.75をかけ、端数を調整して整数にしたもの。例として125BPM (<em><strong>B</strong>eats <strong>P</strong>er
   * <strong>M</strong>inute</em>)における一拍は:
   *
   * <p>93.75[フレーム/秒] / (125[拍/分] / 60[秒/分]) = {@code 45}[フレーム/拍]
   *
   * <p>ここで設定した値は分割された上で{@link FramePhoneme#frameLength}に割り当てられる。どのように分割されるのかについては <a
   * href="https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/song.md#%E5%AD%90%E9%9F%B3%E3%81%AE%E4%BE%B5%E9%A3%9F">子音の侵食</a>を参照。
   */
  @SerializedName("frame_length")
  @Expose
  public long frameLength;

  private Note() {
    lyric = "";
    frameLength = 0;
  }

  public Note(@Nullable String id, @Nullable Byte key, @Nonnull String lyric, long frameLength) {
    this.id = id;
    this.key = key;
    this.lyric = lyric;
    this.frameLength = frameLength;
  }

  /**
   * このインスタンスが不正であるときエラーを返す。
   *
   * <p>不正であるとは、{@code @throws}で示す条件を満たすことである。
   *
   * @throws InvalidQueryException 次のうちどれかを満たす場合
   *     <ul>
   *       <li><a
   *           href="https://voicevox.github.io/voicevox_core/apis/rust_api/voicevox_core/struct.Note.html">Rust
   *           APIの{@code Note}型</a>としてデシリアライズ不可。
   *           <ul>
   *             <li>{@link #key}が負。
   *             <li>{@link #lyric}が空文字列以外の、モーラとして不正な文字列。
   *             <li>{@link #frameLength}が負であるか、もしくは2<sup>53</sup>-1を超過する。
   *           </ul>
   *       <li>{@link #key}が{@code null}かつ{@link #lyric}が空文字列以外。
   *       <li>{@link #key}が非{@code null}かつ{@link #lyric}が空文字列。
   *     </ul>
   */
  public void validate() {
    rsValidate();
  }

  @Override
  public boolean equals(Object obj) {
    if (!(obj instanceof Note)) return false;
    Note other = (Note) obj;

    return (id == null ? other.id == null : id.equals(other.id))
        && (key == null ? other.key == null : key.equals(other.key))
        && lyric.equals(other.lyric)
        && frameLength == other.frameLength;
  }

  @Override
  public Note clone() {
    Note ret = new Note();
    ret.id = id;
    ret.key = key;
    ret.lyric = lyric;
    ret.frameLength = frameLength;
    return ret;
  }

  private native void rsValidate();
}
