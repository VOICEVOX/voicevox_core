package jp.hiroshiba.voicevoxcore.exceptions;

import jp.hiroshiba.voicevoxcore.AudioQuery;

/** {@link AudioQuery}、{@code FrameAudioQuery}、{@code Score}、もしくはその一部が不正。 */
public class InvalidQueryException extends IllegalArgumentException {
  public InvalidQueryException(String message) {
    super(message);
  }

  public InvalidQueryException(String message, Throwable cause) {
    super(message, cause);
  }
}
