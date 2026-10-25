/*
 * 音声合成のテスト。
 * ttsaudioQuery -> synthesisの順に実行する。
 */
package jp.hiroshiba.voicevoxcore.blocking;

import static jp.hiroshiba.voicevoxcore.Wav.wavFromS16le;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrowsExactly;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.ByteArrayOutputStream;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import jp.hiroshiba.voicevoxcore.AccelerationMode;
import jp.hiroshiba.voicevoxcore.AccentPhrase;
import jp.hiroshiba.voicevoxcore.AudioFeature;
import jp.hiroshiba.voicevoxcore.AudioQuery;
import jp.hiroshiba.voicevoxcore.FrameAudioQuery;
import jp.hiroshiba.voicevoxcore.Frames;
import jp.hiroshiba.voicevoxcore.Mora;
import jp.hiroshiba.voicevoxcore.Note;
import jp.hiroshiba.voicevoxcore.Score;
import jp.hiroshiba.voicevoxcore.TestUtils;
import jp.hiroshiba.voicevoxcore.exceptions.InvalidModelDataException;
import jp.hiroshiba.voicevoxcore.exceptions.RunModelException;
import jp.hiroshiba.voicevoxcore.exceptions.StyleNotFoundException;
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

  @Test
  void checkCpuNumThreadsAcceptsUint16() {
    Onnxruntime onnxruntime = loadOnnxruntime();
    OpenJtalk openJtalk = loadOpenJtalk();
    for (int cpuNumThreads : Arrays.asList(0, 1, 2, 4, 0xffff)) {
      Synthesizer.builder(onnxruntime, openJtalk)
          .accelerationMode(AccelerationMode.CPU)
          .cpuNumThreads(cpuNumThreads)
          .build();
    }
  }

  @Test
  void checkCpuNumThreadsDeniesNonUint16() {
    Onnxruntime onnxruntime = loadOnnxruntime();
    OpenJtalk openJtalk = loadOpenJtalk();
    IllegalArgumentException e =
        assertThrowsExactly(
            IllegalArgumentException.class,
            () -> {
              Synthesizer.builder(onnxruntime, openJtalk)
                  .accelerationMode(AccelerationMode.CPU)
                  .cpuNumThreads(0x10000)
                  .build();
            });
    assertEquals("cpuNumThreads", e.getMessage());
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
      synthesizer.loadVoiceModel(model).perform();

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
      synthesizer.loadVoiceModel(model).perform();
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
      synthesizer.loadVoiceModel(model).perform();
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
      synthesizer.loadVoiceModel(model).perform();
    }

    final String TEXT = "こんにちは？";
    int styleId = synthesizer.metas()[0].styles[0].id;

    // FIXME: `interrogativeUpspeak`のデフォルトがJava APIだけ`false`になっている

    byte[] wav1 = synthesizer.tts(TEXT, styleId).perform();

    AudioQuery query = synthesizer.createAudioQuery(TEXT, styleId);
    byte[] wav2 = synthesizer.synthesis(query, styleId).perform();

    List<AccentPhrase> phrases = synthesizer.createAccentPhrases(TEXT, styleId);
    query = AudioQuery.fromAccentPhrases(phrases);
    byte[] wav3 = synthesizer.synthesis(query, styleId).perform();

    phrases = synthesizer.getOpenJtalk().analyze(TEXT);
    phrases = synthesizer.replaceMoraData(phrases, styleId);
    query = AudioQuery.fromAccentPhrases(phrases);
    byte[] wav4 = synthesizer.synthesis(query, styleId).perform();

    phrases = synthesizer.getOpenJtalk().analyze(TEXT);
    phrases = synthesizer.replacePhonemeLength(phrases, styleId);
    phrases = synthesizer.replaceMoraPitch(phrases, styleId);
    query = AudioQuery.fromAccentPhrases(phrases);
    byte[] wav5 = synthesizer.synthesis(query, styleId).perform();

    byte[] wav6 = synthesizer.tts(TEXT, styleId).interrogativeUpspeak(true).perform();

    query = synthesizer.createAudioQuery(TEXT, styleId);
    byte[] wav7 = synthesizer.synthesis(query, styleId).interrogativeUpspeak(true).perform();

    phrases = synthesizer.createAccentPhrases(TEXT, styleId);
    query = AudioQuery.fromAccentPhrases(phrases);
    byte[] wav8 = synthesizer.synthesis(query, styleId).interrogativeUpspeak(true).perform();

    phrases = synthesizer.getOpenJtalk().analyze(TEXT);
    phrases = synthesizer.replaceMoraData(phrases, styleId);
    query = AudioQuery.fromAccentPhrases(phrases);
    byte[] wav9 = synthesizer.synthesis(query, styleId).interrogativeUpspeak(true).perform();

    phrases = synthesizer.getOpenJtalk().analyze(TEXT);
    phrases = synthesizer.replacePhonemeLength(phrases, styleId);
    phrases = synthesizer.replaceMoraPitch(phrases, styleId);
    query = AudioQuery.fromAccentPhrases(phrases);
    byte[] wav10 = synthesizer.synthesis(query, styleId).interrogativeUpspeak(true).perform();

    assertFalse(Arrays.equals(wav1, wav6));
    assertArrayEquals(wav1, wav2);
    assertArrayEquals(wav1, wav3);
    assertArrayEquals(wav1, wav4);
    assertArrayEquals(wav1, wav5);
    assertArrayEquals(wav6, wav7);
    assertArrayEquals(wav6, wav8);
    assertArrayEquals(wav6, wav9);
    assertArrayEquals(wav6, wav10);
  }

  @Test
  void checkStreamTts() throws RunModelException, InvalidModelDataException {
    Onnxruntime onnxruntime = loadOnnxruntime();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(onnxruntime, openJtalk).build();
    try (VoiceModelFile model = openModel()) {
      synthesizer.loadVoiceModel(model).perform();
    }

    final String TEXT = "こんにちは？";
    // `streaming_talk`に対応したスタイルを使用する
    final int STYLE_ID = 302;

    byte[] wav1 = synthesizer.tts(TEXT, STYLE_ID).perform();

    AudioQuery query = synthesizer.createAudioQuery(TEXT, STYLE_ID);
    AudioFeature audioFeature = synthesizer.createAudioFeature(query, STYLE_ID).perform();

    assertEquals(query.frameLength().calculate(), audioFeature.length());

    byte[] pcm = synthesizer.render(audioFeature);
    byte[] wav2 = wavFromS16le(pcm, query.outputSamplingRate, query.outputStereo);

    assertArrayEquals(wav1, wav2);
  }

  private static final String STREAMING_TEXT = "こんにちは？";
  // `streaming_talk`に対応したスタイルを使用する
  private static final int STREAMING_STYLE_ID = 302;

  private Synthesizer buildSynthesizerWithModel()
      throws RunModelException, InvalidModelDataException {
    Synthesizer synthesizer = Synthesizer.builder(loadOnnxruntime(), loadOpenJtalk()).build();
    try (VoiceModelFile model = openModel()) {
      synthesizer.loadVoiceModel(model).perform();
    }
    return synthesizer;
  }

  /** 残りの要素をすべて読み、連結して返す。 */
  private static byte[] drain(SynthesisStream stream) throws RunModelException {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    for (byte[] chunk = stream.next(); chunk != null; chunk = stream.next()) {
      out.write(chunk, 0, chunk.length);
    }
    return out.toByteArray();
  }

  @Test
  void checkStreamingSynthesis() throws RunModelException, InvalidModelDataException {
    Synthesizer synthesizer = buildSynthesizerWithModel();
    AudioQuery query = synthesizer.createAudioQuery(STREAMING_TEXT, STREAMING_STYLE_ID);
    byte[] expected =
        synthesizer.tts(STREAMING_TEXT, STREAMING_STYLE_ID).interrogativeUpspeak(true).perform();

    try (SynthesisStream stream =
        synthesizer.streamingSynthesis(query, STREAMING_STYLE_ID).perform()) {
      // 先頭の要素はWAVヘッダー
      byte[] header = stream.next();
      assertArrayEquals(Arrays.copyOf(expected, 44), header);

      // 区間ごとに合成するため、波形は`tts`と完全には一致しない。Python APIのテストと同様に、
      // ヘッダーと長さだけを確認する。
      assertEquals(expected.length, header.length + drain(stream).length);
      assertNull(stream.next());
    }
  }

  @Test
  void checkStreamingSynthesisSingleSegment() throws RunModelException, InvalidModelDataException {
    Synthesizer synthesizer = buildSynthesizerWithModel();
    AudioQuery query = synthesizer.createAudioQuery(STREAMING_TEXT, STREAMING_STYLE_ID);
    byte[] expected =
        synthesizer.tts(STREAMING_TEXT, STREAMING_STYLE_ID).interrogativeUpspeak(true).perform();

    // 全体が1区間に収まる長さを指定すると、`tts`と完全に一致する
    try (SynthesisStream stream =
        synthesizer.streamingSynthesis(query, STREAMING_STYLE_ID).segmentLength(100.0).perform()) {
      assertEquals(44, stream.next().length);
      byte[] actual = wavFromS16le(drain(stream), query.outputSamplingRate, query.outputStereo);
      assertArrayEquals(expected, actual);
    }
  }

  @Test
  void checkStreamingSynthesisClose() throws RunModelException, InvalidModelDataException {
    Synthesizer synthesizer = buildSynthesizerWithModel();
    AudioQuery query = synthesizer.createAudioQuery(STREAMING_TEXT, STREAMING_STYLE_ID);

    SynthesisStream stream = synthesizer.streamingSynthesis(query, STREAMING_STYLE_ID).perform();
    stream.close();
    stream.close(); // 複数回呼んでもよい
    assertThrowsExactly(IllegalStateException.class, stream::next);
  }

  @Test
  void checkStreamingSynthesisInvalidRange() throws RunModelException, InvalidModelDataException {
    Synthesizer synthesizer = buildSynthesizerWithModel();
    AudioQuery query = synthesizer.createAudioQuery(STREAMING_TEXT, STREAMING_STYLE_ID);

    // 負、NaN、音声の長さを超える値
    for (double startOffset : new double[] {-1.0, Double.NaN, 1.0e6}) {
      assertThrowsExactly(
          IllegalArgumentException.class,
          () ->
              synthesizer
                  .streamingSynthesis(query, STREAMING_STYLE_ID)
                  .startOffset(startOffset)
                  .perform());
    }
    for (double segmentLength : new double[] {0.0, -1.0, Double.NaN}) {
      assertThrowsExactly(
          IllegalArgumentException.class,
          () ->
              synthesizer
                  .streamingSynthesis(query, STREAMING_STYLE_ID)
                  .segmentLength(segmentLength)
                  .perform());
    }
  }

  @Test
  void checkStreamingSynthesisStartOffset() throws RunModelException, InvalidModelDataException {
    Synthesizer synthesizer = buildSynthesizerWithModel();
    AudioQuery query = synthesizer.createAudioQuery(STREAMING_TEXT, STREAMING_STYLE_ID);

    int fullLength;
    try (SynthesisStream stream =
        synthesizer.streamingSynthesis(query, STREAMING_STYLE_ID).perform()) {
      fullLength = stream.next().length + drain(stream).length;
    }

    // 開始位置を進めた分だけ、ヘッダーはそのままで、PCMが短くなる
    try (SynthesisStream stream =
        synthesizer.streamingSynthesis(query, STREAMING_STYLE_ID).startOffset(0.5).perform()) {
      byte[] header = stream.next();
      assertEquals(44, header.length);
      assertTrue(header.length + drain(stream).length < fullLength);
    }
  }

  @Test
  void checkStreamingSynthesisStyleNotFound() throws RunModelException, InvalidModelDataException {
    Synthesizer synthesizer = buildSynthesizerWithModel();
    AudioQuery query = synthesizer.createAudioQuery(STREAMING_TEXT, STREAMING_STYLE_ID);

    // `talk`のスタイル(0、303)と、存在しないスタイルは、`streaming_talk`ではないので見つからない
    for (int styleId : new int[] {0, 303, 999999}) {
      assertThrowsExactly(
          StyleNotFoundException.class,
          () -> synthesizer.streamingSynthesis(query, styleId).perform());
    }
  }

  @Test
  void checkStreamingSynthesisInterrogativeUpspeak()
      throws RunModelException, InvalidModelDataException {
    Synthesizer synthesizer = buildSynthesizerWithModel();
    AudioQuery query = synthesizer.createAudioQuery(STREAMING_TEXT, STREAMING_STYLE_ID);

    int[] lengths = new int[2];
    for (boolean upspeak : new boolean[] {false, true}) {
      byte[] expected =
          synthesizer
              .tts(STREAMING_TEXT, STREAMING_STYLE_ID)
              .interrogativeUpspeak(upspeak)
              .perform();
      try (SynthesisStream stream =
          synthesizer
              .streamingSynthesis(query, STREAMING_STYLE_ID)
              .interrogativeUpspeak(upspeak)
              .perform()) {
        assertEquals(expected.length, stream.next().length + drain(stream).length);
      }
      lengths[upspeak ? 1 : 0] = expected.length;
    }
    // 疑問文の調整によって長さが変わる文なので、オプションが無視されていないことも確かめられる
    assertTrue(lengths[0] < lengths[1]);

    // 既定値は`true`
    try (SynthesisStream stream =
        synthesizer.streamingSynthesis(query, STREAMING_STYLE_ID).perform()) {
      assertEquals(lengths[1], stream.next().length + drain(stream).length);
    }
  }

  @Test
  void checkStreamingSynthesisModelUnloaded() throws RunModelException, InvalidModelDataException {
    Synthesizer synthesizer = Synthesizer.builder(loadOnnxruntime(), loadOpenJtalk()).build();
    try (VoiceModelFile model = openModel()) {
      synthesizer.loadVoiceModel(model).perform();
      AudioQuery query = synthesizer.createAudioQuery(STREAMING_TEXT, STREAMING_STYLE_ID);

      try (SynthesisStream stream =
          synthesizer.streamingSynthesis(query, STREAMING_STYLE_ID).perform()) {
        assertEquals(44, stream.next().length);
        synthesizer.unloadVoiceModel(model.id);
        assertThrowsExactly(StyleNotFoundException.class, stream::next);
      }
    }
  }

  @Test
  void checkAudioFeatureSlicing() throws RunModelException, InvalidModelDataException {
    Onnxruntime onnxruntime = loadOnnxruntime();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(onnxruntime, openJtalk).build();
    try (VoiceModelFile model = openModel()) {
      synthesizer.loadVoiceModel(model).perform();
    }

    final int LEN = 10;
    final int STYLE_ID = 302;

    AudioQuery query = AudioQuery.fromAccentPhrases(Collections.emptyList());
    query.prePhonemeLength = LEN / Frames.FRAME_RATE;
    query.postPhonemeLength = 0.0;
    AudioFeature audioFeature = synthesizer.createAudioFeature(query, STYLE_ID).perform();
    assertEquals(LEN, audioFeature.length());

    List<Integer> reference = Collections.nCopies(LEN, 0);
    for (int i = -1; i <= LEN + 1; i++) {
      for (int j = -1; j <= LEN + 1; j++) {
        for (int k = -1; k <= LEN + 1; k++) {
          for (int l = -1; l <= LEN + 1; l++) {
            int[] expected;
            try {
              List<Integer> slice = reference.subList(i, j).subList(k, l);
              expected = new int[] {i + k, slice.size()};
            } catch (IndexOutOfBoundsException | IllegalArgumentException e) {
              expected = null;
            }

            int[] actual;
            try {
              AudioFeature.Slice slice = audioFeature.slice(i, j).slice(k, l);
              actual = new int[] {slice.getStartOffset(), slice.length()};
            } catch (IndexOutOfBoundsException | IllegalArgumentException e) {
              actual = null;
            }

            assertArrayEquals(expected, actual);
          }
        }
      }
    }
  }

  @Test
  void song() throws RunModelException, InvalidModelDataException {
    Onnxruntime onnxruntime = loadOnnxruntime();
    OpenJtalk openJtalk = loadOpenJtalk();
    Synthesizer synthesizer = Synthesizer.builder(onnxruntime, openJtalk).build();
    try (VoiceModelFile model = openModel()) {
      synthesizer.loadVoiceModel(model).perform();
    }

    final Score SCORE =
        new Score(
            Arrays.asList(
                new Note("①", null, "", 15),
                new Note("②", (byte) 60, "ド", 45),
                new Note("③", (byte) 62, "レ", 45),
                new Note("④", (byte) 64, "ミ", 45),
                new Note("⑤", null, "", 15)));
    final long NUM_TOTAL_FRAMES = SCORE.notes.stream().mapToLong(note -> note.frameLength).sum();

    final int SINGING_TEACHER = 6000;
    final int SINGER = 3000;

    FrameAudioQuery frameAudioQuery = synthesizer.createSingFrameAudioQuery(SCORE, SINGING_TEACHER);

    assertArrayEquals(
        new String[] {"pau", "d", "o", "r", "e", "m", "i", "pau"},
        frameAudioQuery.phonemes.stream().map(p -> p.phoneme).toArray());

    assertArrayEquals(
        new String[] {"①", "②", "②", "③", "③", "④", "④", "⑤"},
        frameAudioQuery.phonemes.stream().map(p -> p.noteId).toArray());

    assertEquals(NUM_TOTAL_FRAMES, frameAudioQuery.f0.length);
    assertEquals(NUM_TOTAL_FRAMES, frameAudioQuery.volume.length);
    assertEquals(
        NUM_TOTAL_FRAMES, frameAudioQuery.phonemes.stream().mapToLong(p -> p.frameLength).sum());

    float[] f0s = synthesizer.createSingFrameF0(SCORE, frameAudioQuery, SINGING_TEACHER);
    assertEquals(NUM_TOTAL_FRAMES, f0s.length);

    float[] volumes = synthesizer.createSingFrameVolume(SCORE, frameAudioQuery, SINGING_TEACHER);
    assertEquals(NUM_TOTAL_FRAMES, volumes.length);

    byte[] wav = synthesizer.frameSynthesis(frameAudioQuery, SINGER);

    assertEquals((byte) 'R', wav[0]);
    assertEquals((byte) 'I', wav[1]);
    assertEquals((byte) 'F', wav[2]);
    assertEquals((byte) 'F', wav[3]);
  }
}
