package jp.hiroshiba.voicevoxcore;

import static org.junit.jupiter.api.Assertions.assertThrowsExactly;

import java.util.Arrays;
import jp.hiroshiba.voicevoxcore.exceptions.IncompatibleQueriesException;
import org.junit.jupiter.api.Test;

class QueriesTest extends TestUtils {
  @Test
  void test() {
    Queries.ensureCompatible(
        new Score(
            Arrays.asList(
                new Note(null, null, "", 0),
                new Note(null, (byte) 0, "ド", 0),
                new Note(null, (byte) 0, "レ", 0),
                new Note(null, (byte) 0, "ミ", 0),
                new Note(null, null, "", 0))),
        new FrameAudioQuery(
            new float[] {},
            new float[] {},
            Arrays.asList(
                new FramePhoneme("pau", 0, null),
                new FramePhoneme("d", 0, null),
                new FramePhoneme("o", 0, null),
                new FramePhoneme("r", 0, null),
                new FramePhoneme("e", 0, null),
                new FramePhoneme("m", 0, null),
                new FramePhoneme("i", 0, null),
                new FramePhoneme("pau", 0, null)),
            1f,
            24000,
            true));

    assertThrowsExactly(
        IncompatibleQueriesException.class,
        () ->
            Queries.ensureCompatible(
                new Score(
                    Arrays.asList(new Note(null, null, "", 0), new Note(null, (byte) 0, "ア", 0))),
                new FrameAudioQuery(
                    new float[] {},
                    new float[] {},
                    Arrays.asList(new FramePhoneme("pau", 0, null), new FramePhoneme("i", 0, null)),
                    1f,
                    24000,
                    true)));
  }
}
