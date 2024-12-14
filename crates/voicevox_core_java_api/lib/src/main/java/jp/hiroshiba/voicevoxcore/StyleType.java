package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;

/** スタイル（style）に対応するモデルの種類。 */
public enum StyleType {
  /** 音声合成クエリの作成と音声合成が可能。 */
  @SerializedName("talk")
  @Expose
  TALK,

  /** 歌唱音声合成用のクエリの作成が可能。 */
  @SerializedName("singing_teacher")
  @Expose
  SINGING_TEACHER,

  /** 歌唱音声合成が可能。 */
  @SerializedName("frame_decode")
  @Expose
  FRAME_DECODE,

  /** 歌唱音声合成用のクエリの作成と歌唱音声合成が可能。 */
  @SerializedName("sing")
  @Expose
  SING,
}
