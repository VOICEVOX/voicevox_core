package jp.hiroshiba.voicevoxcore.exceptions;

/** FrameAudioQueryとScoreの組み合わせが不正。 */
public class IncompatibleQueriesException extends IllegalArgumentException {
  public IncompatibleQueriesException(String message) {
    super(message);
  }

  public IncompatibleQueriesException(String message, Throwable cause) {
    super(message, cause);
  }
}
