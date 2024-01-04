/*
 * 音声合成のテスト。
 * ttsaudioQuery -> synthesisの順に実行する。
 */
package jp.hiroshiba.voicevoxcore;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.SortedMap;
import java.util.TreeMap;
import java.util.stream.Stream;
import jp.hiroshiba.voicevoxcore.Synthesizer.MorphableTargetInfo;
import jp.hiroshiba.voicevoxcore.exceptions.InferenceFailedException;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidModelDataException;
import jp.hiroshiba.voicevoxcore.exceptions.SpeakerFeatureException;
import jp.hiroshiba.voicevoxcore.exceptions.StyleNotFoundException;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.MethodSource;

class SynthesizerTest extends TestUtils {
  @FunctionalInterface
  interface MoraCheckCallback {
    boolean check(Mora mora, Mora otherMora);
  }

  @Test
  void checkIsGpuMode() {
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer =
        Synthesizer.builder(openJtalk).accelerationMode(Synthesizer.AccelerationMode.CPU).build();
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
    VoiceModel model = loadModel();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(openJtalk).build();

    assertTrue(synthesizer.metas().length == 0);

    synthesizer.loadVoiceModel(model);

    assertTrue(synthesizer.metas().length >= 1);
    assertTrue(synthesizer.isLoadedVoiceModel(model.id));

    synthesizer.unloadVoiceModel(model.id);

    assertTrue(synthesizer.metas().length == 0);
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

  @ParameterizedTest
  @MethodSource("morphParamsProvider")
  void checkMorphing(MorphParams params)
      throws InvalidModelDataException, InferenceFailedException {
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer =
        Synthesizer.builder(openJtalk).accelerationMode(Synthesizer.AccelerationMode.CPU).build();

    synthesizer.loadVoiceModel(loadModel());

    int baseStyleId = params.getBaseStyleId();
    AudioQuery query = synthesizer.createAudioQuery("こんにちは", baseStyleId);
    Map<Integer, MorphableTargetInfo> morphableTargets = synthesizer.morphableTargets(baseStyleId);

    for (Map.Entry<Integer, Boolean> entry : params.getTargets().entrySet()) {
      int targetStyleId = entry.getKey();
      boolean shouldSuccess = entry.getValue();

      assertTrue(morphableTargets.get(targetStyleId).isMorphable == shouldSuccess);

      try {
        // TODO: スナップショットテストをやる
        synthesizer.synthesisMorphing(query, baseStyleId, targetStyleId, 0.5);
        assertTrue(shouldSuccess);
      } catch (SpeakerFeatureException e) {
        assertFalse(shouldSuccess);
      }
    }
  }

  static Stream<MorphParams> morphParamsProvider() {
    return Stream.of(
        new MorphParams(
            0,
            new TreeMap<Integer, Boolean>() {
              {
                put(0, false);
                put(1, false);
                put(302, false);
                put(303, false);
              }
            }),
        new MorphParams(
            1,
            new TreeMap<Integer, Boolean>() {
              {
                put(0, false);
                put(1, true);
                put(302, false);
                put(303, false);
              }
            }),
        new MorphParams(
            302,
            new TreeMap<Integer, Boolean>() {
              {
                put(0, false);
                put(1, false);
                put(302, true);
                put(303, true);
              }
            }),
        new MorphParams(
            303,
            new TreeMap<Integer, Boolean>() {
              {
                put(0, false);
                put(1, false);
                put(302, true);
                put(303, true);
              }
            }));
  }

  // TODO: Lombokを使う
  static class MorphParams {
    private final int baseStyleId;
    private final SortedMap<Integer, Boolean> targets;

    MorphParams(int baseStyleId, SortedMap<Integer, Boolean> targets) {
      this.baseStyleId = baseStyleId;
      this.targets = targets;
    }

    int getBaseStyleId() {
      return baseStyleId;
    }

    SortedMap<Integer, Boolean> getTargets() {
      return targets;
    }
  }

  @Test
  void checkMorphableTargetsDeniesUnknownStyle() {
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer =
        Synthesizer.builder(openJtalk).accelerationMode(Synthesizer.AccelerationMode.CPU).build();

    try {
      synthesizer.morphableTargets(0);
    } catch (StyleNotFoundException e) {
    }
  }
}
