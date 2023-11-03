/*
 * VoicevoxCoreInfoのテスト。
 */
package jp.hiroshiba.voicevoxcore;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class InfoTest {
  @Test
  void checkVersion() {
    assertNotNull(VoicevoxCoreInfo.getVersion());
  }


  @Test
  void checkSupportedDevices() {
    VoicevoxCoreInfo.SupportedDevices supportedDevices = VoicevoxCoreInfo.getSupportedDevices();

    assertNotNull(supportedDevices);
    assertTrue(supportedDevices.cpu);
  }
}
