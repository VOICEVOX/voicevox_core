/*
 * GlobalInfoのテスト。
 */
package jp.hiroshiba.voicevoxcore;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class InfoTest {
  @Test
  void checkVersion() {
    assertNotNull(GlobalInfo.getVersion());
  }

  @Test
  void checkSupportedDevices() {
    GlobalInfo.SupportedDevices supportedDevices = GlobalInfo.getSupportedDevices();

    assertNotNull(supportedDevices);
    assertTrue(supportedDevices.cpu);
  }
}
