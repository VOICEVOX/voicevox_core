/*
 * Framesのテスト。
 */
package jp.hiroshiba.voicevoxcore;

import org.junit.jupiter.api.Test;

class FramesTest {
  @Test
  void runStaticBlock() throws ClassNotFoundException {
    Class.forName("jp.hiroshiba.voicevoxcore.Frames");
  }
}
