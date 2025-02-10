package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;

/**
 * <i>スタイル</i>のメタ情報。
 *
 * <p>現在この型はGSONに対応しているが、将来的には <a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
 * target="_blank">Jacksonに切り替わる予定</a> 。
 */
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
   * スタイルの順番。
   *
   * <p>{@link CharacterMeta#styles}の列は、この値に対して昇順に並んでいるべきである。
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
