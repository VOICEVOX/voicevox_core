package jp.hiroshiba.voicevoxcore;

import java.io.File;

class TestUtils {
  VoiceModel loadModel() {
    // cwdはvoicevox_core/crates/voicevox_core_java_api/lib
    String cwd = System.getProperty("user.dir");
    File path = new File(cwd + "/../../test_util/data/model/sample.vvm");

    try {
      return new VoiceModel(path.getCanonicalPath());
    } catch (Exception e) {
      throw new RuntimeException(e);
    }
  }

  Onnxruntime loadOnnxruntime() {
    final String FILENAME = "../../test_util/data/lib/" + Onnxruntime.LIB_VERSIONED_FILENAME;

    try {
      return Onnxruntime.loadOnce().filename(FILENAME).exec();
    } catch (Exception e) {
      throw new RuntimeException(e);
    }
  }

  OpenJtalk loadOpenJtalk() {
    String cwd = System.getProperty("user.dir");
    File path = new File(cwd + "/../../test_util/data/open_jtalk_dic_utf_8-1.11");

    try {
      return new OpenJtalk(path.getCanonicalPath());
    } catch (Exception e) {
      throw new RuntimeException(e);
    }
  }
}
