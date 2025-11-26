package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;
import java.util.ArrayList;
import java.util.List;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidQueryException;

/**
 * AccentPhrase (アクセント句ごとの情報)。
 *
 * <p>この構造体の状態によっては、{@code Synthesizer}の各メソッドは{@link InvalidQueryException}を発する。詳細は{@link
 * #validate}にて。
 *
 * <p>現在この型はGSONに対応しているが、将来的には <a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
 * target="_blank">Jacksonに切り替わる予定</a> 。
 */
public class AccentPhrase {
  /** モーラの配列。 */
  @SerializedName("moras")
  @Expose
  @Nonnull
  public List<Mora> moras;

  /** アクセント箇所。 */
  @SerializedName("accent")
  @Expose
  public int accent;

  /** 後ろに無音を付けるかどうか。 */
  @SerializedName("pause_mora")
  @Expose
  @Nullable
  public Mora pauseMora;

  /** 疑問系かどうか。 */
  @SerializedName("is_interrogative")
  @Expose
  public boolean isInterrogative;

  public AccentPhrase() {
    this.moras = new ArrayList<>();
    this.accent = 0;
    this.pauseMora = null;
    this.isInterrogative = false;
  }

  /**
   * このインスタンスをバリデートする。
   *
   * <p>次のうちどれかを満たすなら{@link InvalidQueryException}を発する。
   *
   * <ul>
   *   <li>{@link #moras}もしくは{@link #pauseMora}の要素のいずれかが不正。
   *   <li>{@link #accent}が{@code 0}。
   * </ul>
   *
   * <p>また次の状態に対してはログで警告を出す。将来的にはエラーになる予定。
   *
   * <ul>
   *   <li>{@link #moras}もしくは{@link #pauseMora}の要素のいずれかが、警告が出る状態。
   *   <li>{@link #accent}が{@link #moras}の数を超過している。
   * </ul>
   */
  public void validate() {
    rsValidate();
  }

  // `moras`の型が`List`のため、`clone`は実装できない

  @Override
  public boolean equals(Object obj) {
    if (!(obj instanceof AccentPhrase)) {
      return false;
    }
    AccentPhrase other = (AccentPhrase) obj;
    return moras.equals(other.moras)
        && accent == other.accent
        && pauseMora == other.pauseMora
        && isInterrogative == other.isInterrogative;
  }

  private native void rsValidate();
}
