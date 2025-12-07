package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;

/**
 * <i>キャラクター</i>のメタ情報。
 *
 * <p>Gsonにおいてはフィールド名はsnake_caseとなる。<a
 * href="https://github.com/VOICEVOX/voicevox_core/blob/main/docs/guide/user/serialization.md"
 * target="_blank">データのシリアライゼーション</a>を参照。
 *
 * <p>Gsonについては将来的には <a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
 * target="_blank">Jacksonに切り替わる予定</a> 。
 */
public class CharacterMeta implements Cloneable {
  /** キャラクター名。 */
  @SerializedName("name")
  @Expose
  @Nonnull
  public final String name;

  /** キャラクターに属するスタイル。 */
  @SerializedName("styles")
  @Expose
  @Nonnull
  public final StyleMeta[] styles;

  /** キャラクターのUUID。 */
  @SerializedName("speaker_uuid")
  @Expose
  @Nonnull
  public final String speakerUuid;

  /** キャラクターのバージョン。 */
  @SerializedName("version")
  @Expose
  @Nonnull
  public final String version;

  /**
   * キャラクターの順番。
   *
   * <p>{@code CharacterMeta}の列は、この値に対して昇順に並んでいるべきである。
   */
  @SerializedName("order")
  @Expose
  @Nullable
  public final Integer order;

  private CharacterMeta() {
    // GSONからコンストラクトするため、このメソッドは呼ばれることは無い。
    // このメソッドは@Nonnullを満たすために必要。
    this.name = "";
    this.styles = new StyleMeta[0];
    this.speakerUuid = "";
    this.version = "";
    this.order = null;
  }

  private CharacterMeta(
      String name, StyleMeta[] styles, String speakerUuid, String version, Integer order) {
    this.name = name;
    this.styles = styles;
    this.speakerUuid = speakerUuid;
    this.version = version;
    this.order = order;
  }

  @Override
  public boolean equals(Object obj) {
    if (!(obj instanceof CharacterMeta)) {
      return false;
    }
    CharacterMeta other = (CharacterMeta) obj;
    return name.equals(other.name)
        && styles.equals(other.styles)
        && speakerUuid.equals(other.speakerUuid)
        && version.equals(other.version)
        && order.equals(other.order);
  }

  @Override
  public CharacterMeta clone() {
    return new CharacterMeta(name, styles.clone(), speakerUuid, version, order);
  }
}
