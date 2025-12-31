package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidQueryException;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/**
 * 音素の情報。
 *
 * <p>このクラスは不正な状態を表現しうる。どのような状態が不正なのかについては{@link #validate}を参照。このクラスを使う関数は、不正な状態に対して{@link
 * InvalidQueryException}をスローする。
 *
 * <p>コンストラクト時には、不正な状態であるかの検証は行われない。外部からのデータが不正でないことを確かめるには、コンストラクト後に{@code validate()}を用いる必要がある。
 *
 * <pre>{@code
 * FramePhoneme framePhoneme = (new Gson()).fromJson(json, FramePhoneme.class);
 * framePhoneme.validate();
 * }</pre>
 *
 * <p>Gsonにおいてはフィールド名はsnake_caseとなる。<a
 * href="https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md"
 * target="_blank">データのシリアライゼーション</a>を参照。
 *
 * <p>Gsonについては将来的には <a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
 * target="_blank">Jacksonに切り替わる予定</a> 。
 */
public final class FramePhoneme implements Cloneable {
  static {
    Dll.loadLibrary();
  }

  /** 音素。 */
  @Expose
  @Nonnull
  @SuppressWarnings("NullableProblems")
  public String phoneme;

  /** 音素のフレーム長。 */
  @SerializedName("frame_length")
  @Expose
  public long frameLength;

  /** 音符のID。 */
  @SerializedName("note_id")
  @Expose
  @Nullable
  public String noteId;

  private FramePhoneme() {
    phoneme = "";
    frameLength = 0;
  }

  public FramePhoneme(@Nonnull String phoneme, long frameLength, @Nullable String noteId) {
    this.phoneme = phoneme;
    this.frameLength = frameLength;
    this.noteId = noteId;
  }

  /**
   * このインスタンスが不正であるときエラーを返す。
   *
   * <p>不正であるとは、{@code @throws}で示す条件を満たすことである。
   *
   * @throws InvalidQueryException 次のうちどれかを満たす場合
   *     <ul>
   *       <li><a
   *           href="https://voicevox.github.io/voicevox_core/apis/rust_api/voicevox_core/struct.FramePhoneme.html">Rust
   *           APIの{@code FramePhoneme}型</a>としてデシリアライズ不可。
   *           <ul>
   *             <li>{@link #phoneme}が音素として不正。
   *             <li>{@link #frameLength}が負であるか、もしくは2<sup>53</sup>-1を超過する。
   *           </ul>
   *     </ul>
   */
  public void validate() {
    rsValidate();
  }

  @Override
  public boolean equals(Object obj) {
    if (!(obj instanceof FramePhoneme)) return false;
    FramePhoneme other = (FramePhoneme) obj;

    return phoneme.equals(other.phoneme)
        && frameLength == other.frameLength
        && (noteId == null ? other.noteId == null : noteId.equals(other.noteId));
  }

  @Override
  public FramePhoneme clone() {
    FramePhoneme ret = new FramePhoneme();
    ret.phoneme = phoneme;
    ret.frameLength = frameLength;
    ret.noteId = noteId;
    return ret;
  }

  private native void rsValidate();
}
