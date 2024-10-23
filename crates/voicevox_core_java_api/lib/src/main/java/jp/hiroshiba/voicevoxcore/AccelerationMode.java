package jp.hiroshiba.voicevoxcore;

/** ハードウェアアクセラレーションモード。 */
public enum AccelerationMode {
  /** 実行環境に合わせて自動的に選択する。 */
  AUTO,
  /** CPUに設定する。 */
  CPU,
  /** GPUに設定する。 */
  GPU,
}
