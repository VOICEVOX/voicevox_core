package jp.Hiroshiba.VoicevoxCore;

import java.nio.file.Path;
import java.nio.file.Paths;

class Utils {
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
}
