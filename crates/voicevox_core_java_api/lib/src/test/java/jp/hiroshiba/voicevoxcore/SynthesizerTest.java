/*
 * 音声合成のテスト。
 * ttsaudioQuery -> synthesisの順に実行する。
 */
package jp.hiroshiba.voicevoxcore;

import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
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
  void checkModel() {
    VoiceModel model = loadModel();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer =
        Synthesizer.builder(openJtalk).accelerationMode(Synthesizer.AccelerationMode.CPU).build();

    synthesizer.loadVoiceModel(model);

    assertTrue(synthesizer.isLoadedVoiceModel(model.id));
  }

  @Test
  void checkAudioQuery() {
    VoiceModel model = loadModel();
    Synthesizer synthesizer = createSynthesizer();

    AudioQuery query = synthesizer.createAudioQuery("こんにちは", model.metas[0].styles[0].id).execute();

    synthesizer.synthesis(query, model.metas[0].styles[0].id).execute();
  }

  @Test
  void checkAccentPhrases() {
    VoiceModel model = loadModel();
    Synthesizer synthesizer = createSynthesizer();

    List<AccentPhrase> accentPhrases =
        synthesizer.createAccentPhrases("こんにちは", model.metas[0].styles[0].id).execute();
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
  void checkTts() {
    VoiceModel model = loadModel();
    Synthesizer synthesizer = createSynthesizer();
    synthesizer.tts("こんにちは", model.metas[0].styles[0].id).execute();
  }
}
