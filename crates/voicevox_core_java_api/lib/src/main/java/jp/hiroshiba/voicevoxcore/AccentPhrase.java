package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;
import java.util.ArrayList;
import java.util.List;

/**
 * AccentPhrase (アクセント句ごとの情報)。
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
}
