package jp.hiroshiba.voicevoxcore;

/** ハードウェアアクセラレーションモード。 */
public class AccelerationMode {
  /** 実行環境に合わせて自動的に選択する。 */
  public static final AccelerationMode AUTO = new AccelerationMode("AUTO");

  /** CPUに設定する。 */
  public static final AccelerationMode CPU = new AccelerationMode("CPU");

  /** GPUに設定する。 */
  public static final AccelerationMode GPU = new AccelerationMode("GPU");

  private final String identifier;

  private AccelerationMode(String identifier) {
    this.identifier = identifier;
  }

  @Override
  public String toString() {
    return identifier;
  }
}
