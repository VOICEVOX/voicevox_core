package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;

/** スタイル（style）のメタ情報。 */
public class StyleMeta {
  /** スタイル名。 */
  @SerializedName("name")
  @Expose
  @Nonnull
  public final String name;

  /** スタイルID。 */
  @SerializedName("id")
  @Expose
  public final int id;

  /** スタイルに対応するモデルの種類。 */
  @SerializedName("type")
  @Expose
  @Nonnull
  public final StyleType type;

  /**
   * 話者の順番。
   *
   * <p>{@link SpeakerMeta#styles}の列は、この値に対して昇順に並んでいるべきである。
   */
  @SerializedName("order")
  @Expose
  @Nullable
  public final Integer order;

  private StyleMeta() {
    this.name = "";
    this.id = 0;
    this.type = StyleType.TALK;
    this.order = null;
  }
}
