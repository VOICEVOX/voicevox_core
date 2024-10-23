package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;

/** 話者（speaker）のメタ情報。 */
public class SpeakerMeta {
  /** 話者名。 */
  @SerializedName("name")
  @Expose
  @Nonnull
  public final String name;

  /** 話者に属するスタイル。 */
  @SerializedName("styles")
  @Expose
  @Nonnull
  public final StyleMeta[] styles;

  /** 話者のUUID。 */
  @SerializedName("speaker_uuid")
  @Expose
  @Nonnull
  public final String speakerUuid;

  /** 話者のバージョン。 */
  @SerializedName("version")
  @Expose
  @Nonnull
  public final String version;

  /**
   * 話者の順番。
   *
   * <p>{@code SpeakerMeta}の列は、この値に対して昇順に並んでいるべきである。
   */
  @SerializedName("order")
  @Expose
  @Nullable
  public final Integer order;

  private SpeakerMeta() {
    // GSONからコンストラクトするため、このメソッドは呼ばれることは無い。
    // このメソッドは@Nonnullを満たすために必要。
    this.name = "";
    this.styles = new StyleMeta[0];
    this.speakerUuid = "";
    this.version = "";
    this.order = null;
  }
}
