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
 * <p>このクラスは不正な状態を表現しうる。どのような状態が不正なのかについては{@link #validate}を参照。このクラスを使う関数は、不正な状態に対して{@link
 * InvalidQueryException}をスローする。
 *
 * <p>コンストラクト時には、不正な状態であるかの検証は行われない。外部からのデータが不正でないことを確かめるには、コンストラクト後に{@code validate()}を用いる必要がある。
 *
 * <pre>{@code
 * AccentPhrase accentPhrase = (new Gson()).fromJson(json, AccentPhrase.class);
 * accentPhrase.validate();
 * }</pre>
 *
 * <p>Gsonにおいてはフィールド名はsnake_caseとなる。<a
 * href="https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md"
 * target="_blank">データのシリアライゼーション</a>を参照。
 *
 * <p>Gsonについては将来的には <a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
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
   * このインスタンスが不正であるときエラーを返す。
   *
   * <p>不正であるとは、{@code @throws}で示す条件を満たすことである。
   *
   * <p>また次の状態に対してはログで警告を出す。将来的にはエラーになる予定。
   *
   * <ul>
   *   <li>{@link #moras}もしくは{@link #pauseMora}の要素のいずれかが、警告が出る状態。
   *   <li>{@link #accent}が{@link #moras}の数を超過している。
   * </ul>
   *
   * @throws InvalidQueryException 次のうちどれかを満たす場合
   *     <ul>
   *       <li><a
   *           href="https://voicevox.github.io/voicevox_core/apis/rust_api/voicevox_core/struct.AccentPhrase.html">Rust
   *           APIの{@code AccentPhrase}型</a>としてデシリアライズ不可。
   *           <ul>
   *             <li>{@link #accent}が負であるか、もしくは32ビットプラットフォームの場合2<sup>32</sup>-1を超過する。
   *           </ul>
   *       <li>{@link #moras}もしくは{@link #pauseMora}の要素のいずれかが不正。
   *       <li>{@link #accent}が{@code 0}。
   *     </ul>
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
