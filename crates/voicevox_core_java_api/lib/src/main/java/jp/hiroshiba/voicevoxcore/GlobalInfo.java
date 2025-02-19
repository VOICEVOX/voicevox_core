package jp.hiroshiba.voicevoxcore;

import jakarta.annotation.Nonnull;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/** VOICEVOX CORE自体の情報。 */
public class GlobalInfo {
  static {
    Dll.loadLibrary();
  }

  /**
   * ライブラリのバージョン。
   *
   * @return ライブラリのバージョン。
   */
  @Nonnull
  public static String getVersion() {
    return rsGetVersion();
  }

  @Nonnull
  private static native String rsGetVersion();

  // FIXME: dead code
  @Nonnull
  private static native String rsGetSupportedDevicesJson();
}
