/*
 * GlobalInfoのテスト。
 */
package jp.hiroshiba.voicevoxcore;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class InfoTest extends TestUtils {
  @Test
  void checkVersion() {
    assertNotNull(GlobalInfo.getVersion());
  }

  // TODO: 別の場所に移す
  @Test
  void checkSupportedDevices() {
    GlobalInfo.SupportedDevices supportedDevices = loadOnnxruntime().supportedDevices();

    assertNotNull(supportedDevices);
    assertTrue(supportedDevices.cpu);
  }
}
