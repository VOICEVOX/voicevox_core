package jp.hiroshiba.voicevoxcore;

import java.io.File;

class TestUtils {
  VoiceModel voiceModel = null;
  OpenJtalk openJtalk = null;
  Synthesizer synthesizer = null;

  VoiceModel loadModel() {
    if (voiceModel != null) {
      return voiceModel;
    }
    // cwdはvoicevox_core/crates/voicevox_core_java_api/lib
    String cwd = System.getProperty("user.dir");
    File path = new File(cwd + "/../../../model/sample.vvm");

    try {
      return new VoiceModel(path.getCanonicalPath());
    } catch (Exception e) {
      throw new RuntimeException(e);
    }
  }

  OpenJtalk loadOpenJtalk() {
    if (openJtalk != null) {
      return openJtalk;
    }
    String cwd = System.getProperty("user.dir");
    File path = new File(cwd + "/../../test_util/data/open_jtalk_dic_utf_8-1.11");

    try {
      return new OpenJtalk(path.getCanonicalPath());
    } catch (Exception e) {
      throw new RuntimeException(e);
    }
  }

  Synthesizer createSynthesizer() {
    if (synthesizer != null) {
      return synthesizer;
    }
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer =
        Synthesizer.builder(openJtalk).accelerationMode(Synthesizer.AccelerationMode.CPU).build();
    synthesizer.loadVoiceModel(loadModel());
    return synthesizer;
  }
}
