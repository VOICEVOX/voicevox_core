package jp.hiroshiba.voicevoxcore.exceptions;

/** AudioQuery、もしくはその一部が不正。 */
public class InvalidQueryException extends IllegalArgumentException {
  public InvalidQueryException(String message) {
    super(message);
  }

  public InvalidQueryException(String message, Throwable cause) {
    super(message, cause);
  }
}
