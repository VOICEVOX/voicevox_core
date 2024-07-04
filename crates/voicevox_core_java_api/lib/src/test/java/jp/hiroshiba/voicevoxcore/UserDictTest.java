package jp.hiroshiba.voicevoxcore;

import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Files;
import java.nio.file.Path;
import jp.hiroshiba.voicevoxcore.exceptions.InferenceFailedException;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidModelDataException;
import jp.hiroshiba.voicevoxcore.exceptions.LoadUserDictException;
import org.junit.jupiter.api.Test;

class UserDictTest extends TestUtils {

  // 辞書ロードのテスト。
  // 辞書ロード前後でkanaが異なることを確認する
  @Test
  void checkLoad()
      throws InferenceFailedException, InvalidModelDataException, LoadUserDictException {
    VoiceModel model = loadModel();
    Onnxruntime onnxruntime = loadOnnxruntime();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(onnxruntime, openJtalk).build();
    UserDict userDict = new UserDict();
    synthesizer.loadVoiceModel(model);
    AudioQuery query1 =
        synthesizer.createAudioQuery(
            "this_word_should_not_exist_in_default_dictionary", model.metas[0].styles[0].id);

    userDict.addWord(new UserDict.Word("this_word_should_not_exist_in_default_dictionary", "テスト"));
    openJtalk.useUserDict(userDict);
    AudioQuery query2 =
        synthesizer.createAudioQuery(
            "this_word_should_not_exist_in_default_dictionary", model.metas[0].styles[0].id);
    assertTrue(query1.kana != query2.kana);
  }

  // 辞書操作のテスト。
  @Test
  void checkManipulation() throws Exception {
    UserDict userDict = new UserDict();
    // 単語追加
    String uuid = userDict.addWord(new UserDict.Word("hoge", "ホゲ"));
    assertTrue(userDict.toHashMap().get(uuid) != null);

    // 単語更新
    userDict.updateWord(uuid, new UserDict.Word("hoge", "ホゲホゲ"));
    assertTrue(userDict.toHashMap().get(uuid).pronunciation.equals("ホゲホゲ"));

    // 単語削除
    userDict.removeWord(uuid);
    assertTrue(userDict.toHashMap().get(uuid) == null);

    // 辞書のインポート
    userDict.addWord(new UserDict.Word("hoge", "ホゲ"));
    UserDict userDict2 = new UserDict();
    userDict2.addWord(new UserDict.Word("fuga", "フガ"));
    userDict.importDict(userDict2);
    assertTrue(userDict.toHashMap().size() == 2);

    // 辞書の保存/読み込み
    Path path = Files.createTempFile("voicevox_user_dict", ".json");
    try {
      UserDict userDict3 = new UserDict();
      userDict.save(path.toString());
      userDict3.load(path.toString());
      assertTrue(userDict3.toHashMap().size() == 2);
    } finally {
      Files.deleteIfExists(path);
    }
  }
}
