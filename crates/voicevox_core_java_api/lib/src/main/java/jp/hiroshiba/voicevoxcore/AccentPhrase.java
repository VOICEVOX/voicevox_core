package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import java.util.ArrayList;
import java.util.List;
import javax.annotation.Nonnull;
import javax.annotation.Nullable;

/** AccentPhrase (アクセント句ごとの情報)。 */
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
}
