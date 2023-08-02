/*
 * 音声合成のテスト。
 * ttsaudioQuery -> synthesisの順に実行する。
 */
package jp.Hiroshiba.VoicevoxCore;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.EnumSet;

@Timeout(10)
class SynthesizerTest {
  VoiceModel model() {
    // cwdはvoicevox_core/crates/voicevox_core_java_api/lib
    String cwd = System.getProperty("user.dir");
    Path path = Paths.get(cwd, "..", "..", "..", "model", "sample.vvm");

    return new VoiceModel(path.toString());

  }

  OpenJtalk openJtalk() {
    String cwd = System.getProperty("user.dir");
    Path path = Paths.get(cwd, "..", "..", "test_util", "data", "open_jtalk_dic_utf_8-1.11");

    return new OpenJtalk(path.toString());
  }

  @Test
  void checkModel() {
    try (VoiceModel model = model();
        OpenJtalk openJtalk = openJtalk();
        Synthesizer synthesizer = Synthesizer.builder(openJtalk).build()) {
      synthesizer.loadVoiceModel(model);
      assertTrue(synthesizer.isLoadedVoiceModel(model.id));
      synthesizer.unloadVoiceModel(model.id);
      assertFalse(synthesizer.isLoadedVoiceModel(model.id));
    }
  }

  @Test
  void checkAudioQuery() {
    try (VoiceModel model = model();
        OpenJtalk openJtalk = openJtalk();
        Synthesizer synthesizer = Synthesizer.builder(openJtalk).build()) {
      synthesizer.loadVoiceModel(model);
      AudioQuery query = synthesizer.audioQuery("こんにちは", model.metas[0].styles[0].id,
          EnumSet.noneOf(Synthesizer.AudioQueryOption.class));

      synthesizer.synthesis(query, model.metas[0].styles[0].id, EnumSet.noneOf(Synthesizer.SynthesisOption.class));
    }
  }

  @Test
  void checkTts() {
    try (VoiceModel model = model();
        OpenJtalk openJtalk = openJtalk();
        Synthesizer synthesizer = Synthesizer.builder(openJtalk).build()) {
      synthesizer.loadVoiceModel(model);
      synthesizer.tts("こんにちは", model.metas[0].styles[0].id, EnumSet.noneOf(Synthesizer.TtsOption.class));
    }
  }
}
