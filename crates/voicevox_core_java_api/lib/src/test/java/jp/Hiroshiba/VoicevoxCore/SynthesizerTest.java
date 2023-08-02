/*
 * 音声合成のテスト。
 * ttsaudioQuery -> synthesisの順に実行する。
 */
package jp.Hiroshiba.VoicevoxCore;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.File;

class SynthesizerTest {
  VoiceModel model() {
    // cwdはvoicevox_core/crates/voicevox_core_java_api/lib
    String cwd = System.getProperty("user.dir");
    File path = new File(
        cwd + "/../../../model/sample.vvm");

    return new VoiceModel(path.getAbsolutePath());

  }

  OpenJtalk openJtalk() {
    String cwd = System.getProperty("user.dir");
    File path = new File(
        cwd + "/../../test_util/data/open_jtalk_dic_utf_8-1.11");
    return new OpenJtalk(path.getAbsolutePath());
  }

  @Test
  void checkModel() {
    try (VoiceModel model = model();
        OpenJtalk openJtalk = new OpenJtalk();
        Synthesizer synthesizer = Synthesizer.builder(openJtalk).build()) {
      synthesizer.loadVoiceModel(model);
      assertTrue(synthesizer.isLoadedVoiceModel(model.id));
      synthesizer.unloadVoiceModel(model.id);
      assertFalse(synthesizer.isLoadedVoiceModel(model.id));
    }
  }

  @Test
  void checkTts() {
    try (VoiceModel model = model();
        OpenJtalk openJtalk = new OpenJtalk();
        Synthesizer synthesizer = Synthesizer.builder(openJtalk).build()) {
      synthesizer.loadVoiceModel(model);
    }
  }
}
