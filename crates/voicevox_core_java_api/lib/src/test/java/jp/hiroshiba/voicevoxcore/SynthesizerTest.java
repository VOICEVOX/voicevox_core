/*
 * 音声合成のテスト。
 * ttsaudioQuery -> synthesisの順に実行する。
 */
package jp.hiroshiba.voicevoxcore;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import jp.hiroshiba.voicevoxcore.exceptions.InferenceFailedException;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidModelDataException;
import org.junit.jupiter.api.Test;

class SynthesizerTest extends TestUtils {
  @FunctionalInterface
  interface MoraCheckCallback {
    boolean check(Mora mora, Mora otherMora);
  }

  boolean checkAllMoras(
      List<AccentPhrase> accentPhrases,
      List<AccentPhrase> otherAccentPhrases,
      MoraCheckCallback checker) {
    for (int i = 0; i < accentPhrases.size(); i++) {
      AccentPhrase accentPhrase = accentPhrases.get(i);
      for (int j = 0; j < accentPhrase.moras.size(); j++) {
        Mora mora = accentPhrase.moras.get(j);
        Mora otherMora = otherAccentPhrases.get(i).moras.get(j);
        if (!checker.check(mora, otherMora)) {
          return false;
        }
      }
    }
    return true;
  }

  @Test
  void checkModel() throws InvalidModelDataException {
    VoiceModel model = loadModel();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(openJtalk).build();
    synthesizer.loadVoiceModel(model);
    assertTrue(synthesizer.isLoadedVoiceModel(model.id));
    synthesizer.unloadVoiceModel(model.id);
    assertFalse(synthesizer.isLoadedVoiceModel(model.id));
  }

  @Test
  void checkAudioQuery() throws InferenceFailedException, InvalidModelDataException {
    VoiceModel model = loadModel();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(openJtalk).build();
    synthesizer.loadVoiceModel(model);
    AudioQuery query = synthesizer.createAudioQuery("こんにちは", model.metas[0].styles[0].id);

    synthesizer.synthesis(query, model.metas[0].styles[0].id).execute();
  }

  @Test
  void checkAccentPhrases() throws InferenceFailedException, InvalidModelDataException {
    VoiceModel model = loadModel();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(openJtalk).build();
    synthesizer.loadVoiceModel(model);
    List<AccentPhrase> accentPhrases =
        synthesizer.createAccentPhrases("こんにちは", model.metas[0].styles[0].id);
    List<AccentPhrase> accentPhrases2 =
        synthesizer.replaceMoraPitch(accentPhrases, model.metas[1].styles[0].id);
    assertTrue(
        checkAllMoras(
            accentPhrases, accentPhrases2, (mora, otherMora) -> mora.pitch != otherMora.pitch));
    List<AccentPhrase> accentPhrases3 =
        synthesizer.replacePhonemeLength(accentPhrases, model.metas[1].styles[0].id);
    assertTrue(
        checkAllMoras(
            accentPhrases,
            accentPhrases3,
            (mora, otherMora) -> mora.vowelLength != otherMora.vowelLength));
    List<AccentPhrase> accentPhrases4 =
        synthesizer.replaceMoraData(accentPhrases, model.metas[1].styles[0].id);
    assertTrue(
        checkAllMoras(
            accentPhrases,
            accentPhrases4,
            (mora, otherMora) ->
                mora.pitch != otherMora.pitch && mora.vowelLength != otherMora.vowelLength));
  }

  @Test
  void checkTts() throws InferenceFailedException, InvalidModelDataException {
    VoiceModel model = loadModel();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(openJtalk).build();
    synthesizer.loadVoiceModel(model);
    synthesizer.tts("こんにちは", model.metas[0].styles[0].id);
  }
}
