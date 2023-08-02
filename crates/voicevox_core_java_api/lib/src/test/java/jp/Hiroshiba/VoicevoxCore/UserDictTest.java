package jp.Hiroshiba.VoicevoxCore;

import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.EnumSet;

import org.junit.jupiter.api.Test;

class UserDictTest
    extends Utils {

  @Test
  void checkLoad() {
    try (VoiceModel model = model();
        OpenJtalk openJtalk = openJtalk();
        Synthesizer synthesizer = Synthesizer.builder(openJtalk).build();
        UserDict userDict = new UserDict()) {
      synthesizer.loadVoiceModel(model);
      AudioQuery query1 = synthesizer.audioQuery("this_word_should_not_exist_in_default_dictionary",
          model.metas[0].styles[0].id,
          EnumSet.noneOf(Synthesizer.AudioQueryOption.class));
      userDict.addWord(
          new UserDict.Word("this_word_should_not_exist_in_default_dictionary", "テスト"));
      openJtalk.useUserDict(userDict);
      AudioQuery query2 = synthesizer.audioQuery("this_word_should_not_exist_in_default_dictionary",
          model.metas[0].styles[0].id,
          EnumSet.noneOf(Synthesizer.AudioQueryOption.class));
      assertTrue(query1.kana != query2.kana);

    }
  }
}
