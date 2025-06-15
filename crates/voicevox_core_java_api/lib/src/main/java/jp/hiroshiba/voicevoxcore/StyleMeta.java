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
public class StyleMeta implements Cloneable {
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

  private StyleMeta(String name, int id, StyleType type, Integer order) {
    this.name = name;
    this.id = id;
    this.type = type;
    this.order = order;
  }

  @Override
  public boolean equals(Object obj) {
    if (!(obj instanceof StyleMeta)) {
      return false;
    }
    StyleMeta other = (StyleMeta) obj;
    return name.equals(other.name)
        && id == other.id
        && type == other.type
        && order.equals(other.order);
  }

  @Override
  public StyleMeta clone() {
    return new StyleMeta(name, id, type, order);
  }
}
