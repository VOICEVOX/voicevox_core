package jp.Hiroshiba.VoicevoxCore;

import java.util.List;
import java.util.ArrayList;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;

/** AccentPhrase (アクセント句ごとの情報)。 */
public class AccentPhrase {
  /** モーラの配列。 */
  @JsonProperty("moras")
  @SerializedName("moras")
  @Expose
  @Nonnull
  public List<Mora> moras;

  /** アクセント箇所。 */
  @JsonProperty("accent")
  @SerializedName("accent")
  @Expose
  public int accent;

  /** 後ろに無音を付けるかどうか。 */
  @JsonProperty("pause_mora")
  @SerializedName("pause_mora")
  @Expose
  @Nullable
  public Mora pauseMora;

  /** 疑問系かどうか。 */
  @JsonProperty("is_interrogative")
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
