/*
 * 音声合成のテスト。
 * ttsaudioQuery -> synthesisの順に実行する。
 */
package jp.hiroshiba.voicevoxcore.blocking;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import jp.hiroshiba.voicevoxcore.AccelerationMode;
import jp.hiroshiba.voicevoxcore.AccentPhrase;
import jp.hiroshiba.voicevoxcore.AudioQuery;
import jp.hiroshiba.voicevoxcore.Mora;
import jp.hiroshiba.voicevoxcore.TestUtils;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidModelDataException;
import jp.hiroshiba.voicevoxcore.exceptions.RunModelException;
import org.junit.jupiter.api.Test;

class SynthesizerTest extends TestUtils {
  @FunctionalInterface
  interface MoraCheckCallback {
    boolean check(Mora mora, Mora otherMora);
  }

  @Test
  void checkIsGpuMode() {
    Onnxruntime onnxruntime = loadOnnxruntime();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer =
        Synthesizer.builder(onnxruntime, openJtalk).accelerationMode(AccelerationMode.CPU).build();
    assertFalse(synthesizer.isGpuMode());
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
    Onnxruntime onnxruntime = loadOnnxruntime();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(onnxruntime, openJtalk).build();

    assertTrue(synthesizer.metas().length == 0);

    try (VoiceModelFile model = openModel()) {
      synthesizer.loadVoiceModel(model);

      assertTrue(synthesizer.metas().length >= 1);
      assertTrue(synthesizer.isLoadedVoiceModel(model.id));

      synthesizer.unloadVoiceModel(model.id);

      assertTrue(synthesizer.metas().length == 0);
      assertFalse(synthesizer.isLoadedVoiceModel(model.id));
    }
  }

  @Test
  void checkAudioQuery() throws RunModelException, InvalidModelDataException {
    Onnxruntime onnxruntime = loadOnnxruntime();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(onnxruntime, openJtalk).build();

    try (VoiceModelFile model = openModel()) {
      synthesizer.loadVoiceModel(model);
    }

    AudioQuery query = synthesizer.createAudioQuery("こんにちは", synthesizer.metas()[0].styles[0].id);
    synthesizer.synthesis(query, synthesizer.metas()[0].styles[0].id).perform();
  }

  @Test
  void checkAccentPhrases() throws RunModelException, InvalidModelDataException {
    OpenJtalk openJtalk = loadOpenJtalk();
    Onnxruntime onnxruntime = loadOnnxruntime();
    Synthesizer synthesizer = Synthesizer.builder(onnxruntime, openJtalk).build();
    try (VoiceModelFile model = openModel()) {
      synthesizer.loadVoiceModel(model);
    }
    List<AccentPhrase> accentPhrases =
        synthesizer.createAccentPhrases("こんにちは", synthesizer.metas()[0].styles[0].id);
    List<AccentPhrase> accentPhrases2 =
        synthesizer.replaceMoraPitch(accentPhrases, synthesizer.metas()[1].styles[0].id);
    assertTrue(
        checkAllMoras(
            accentPhrases, accentPhrases2, (mora, otherMora) -> mora.pitch != otherMora.pitch));
    List<AccentPhrase> accentPhrases3 =
        synthesizer.replacePhonemeLength(accentPhrases, synthesizer.metas()[1].styles[0].id);
    assertTrue(
        checkAllMoras(
            accentPhrases,
            accentPhrases3,
            (mora, otherMora) -> mora.vowelLength != otherMora.vowelLength));
    List<AccentPhrase> accentPhrases4 =
        synthesizer.replaceMoraData(accentPhrases, synthesizer.metas()[1].styles[0].id);
    assertTrue(
        checkAllMoras(
            accentPhrases,
            accentPhrases4,
            (mora, otherMora) ->
                mora.pitch != otherMora.pitch && mora.vowelLength != otherMora.vowelLength));
  }

  @Test
  void checkTts() throws RunModelException, InvalidModelDataException {
    Onnxruntime onnxruntime = loadOnnxruntime();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(onnxruntime, openJtalk).build();
    try (VoiceModelFile model = openModel()) {
      synthesizer.loadVoiceModel(model);
    }
    synthesizer.tts("こんにちは", synthesizer.metas()[0].styles[0].id).perform();
  }
}
