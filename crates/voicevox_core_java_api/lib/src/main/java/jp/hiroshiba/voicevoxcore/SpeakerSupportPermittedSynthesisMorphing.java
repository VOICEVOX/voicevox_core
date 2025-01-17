package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;

/** モーフィング機能への対応。 */
public enum SpeakerSupportPermittedSynthesisMorphing {
  /** 全て許可。 */
  @Expose
  ALL,

  /** 同じ話者内でのみ許可。 */
  @Expose
  SELF_ONLY,

  /** 全て禁止。 */
  @Expose
  NOTHING,
}
