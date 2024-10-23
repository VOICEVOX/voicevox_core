package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;

/** スタイル（style）に対応するモデルの種類。 */
public enum StyleType {
  /** 音声合成クエリの作成と音声合成が可能。 */
  @SerializedName("talk")
  @Expose
  TALK,
}
