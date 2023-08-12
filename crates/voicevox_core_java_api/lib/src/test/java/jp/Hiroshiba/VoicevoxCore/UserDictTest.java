package jp.Hiroshiba.VoicevoxCore;

import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.EnumSet;
import org.junit.jupiter.api.Test;

class UserDictTest extends TestUtils {

  // 辞書ロードのテスト。
  // 辞書ロード前後でkanaが異なることを確認する
  @Test
  void checkLoad() {
    VoiceModel model = loadModel();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(openJtalk).build();
    UserDict userDict = new UserDict();
    synthesizer.loadVoiceModel(model);
    AudioQuery query1 =
        synthesizer.audioQuery(
            "this_word_should_not_exist_in_default_dictionary",
            model.metas[0].styles[0].id,
            EnumSet.noneOf(Synthesizer.AudioQueryOption.class));
    userDict.addWord(new UserDict.Word("this_word_should_not_exist_in_default_dictionary", "テスト"));
    openJtalk.useUserDict(userDict);
    AudioQuery query2 =
        synthesizer.audioQuery(
            "this_word_should_not_exist_in_default_dictionary",
            model.metas[0].styles[0].id,
            EnumSet.noneOf(Synthesizer.AudioQueryOption.class));
    assertTrue(query1.kana != query2.kana);
  }

  // 辞書操作のテスト。
  @Test
  void checkManipulation() throws Exception {
    UserDict userDict = new UserDict();
    // 単語追加
    String uuid = userDict.addWord(new UserDict.Word("hoge", "ホゲ"));
    assertTrue(userDict.getWords().get(uuid) != null);

    // 単語更新
    userDict.updateWord(uuid, new UserDict.Word("hoge", "ホゲホゲ"));
    assertTrue(userDict.getWords().get(uuid).pronunciation.equals("ホゲホゲ"));

    // 単語削除
    userDict.removeWord(uuid);
    assertTrue(userDict.getWords().get(uuid) == null);

    // 辞書のインポート
    userDict.addWord(new UserDict.Word("hoge", "ホゲ"));
    UserDict userDict2 = new UserDict();
    userDict2.addWord(new UserDict.Word("fuga", "フガ"));
    userDict.importDict(userDict2);
    assertTrue(userDict.getWords().size() == 2);

    // 辞書の保存/読み込み
    Path path = Files.createTempFile("voicevox_user_dict", ".json");
    try {
      UserDict userDict3 = new UserDict();
      userDict.save(path.toString());
      userDict3.load(path.toString());
      assertTrue(userDict3.getWords().size() == 2);
    } finally {
      Files.deleteIfExists(path);
    }
  }
}
