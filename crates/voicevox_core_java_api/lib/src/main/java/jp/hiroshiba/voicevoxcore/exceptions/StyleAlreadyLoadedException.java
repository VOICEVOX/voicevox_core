package jp.hiroshiba.voicevoxcore.exceptions;

/** すでに読み込まれているスタイルを読み込もうとした。 */
public class StyleAlreadyLoadedException extends IllegalStateException {
  public StyleAlreadyLoadedException(String message) {
    super(message);
  }

  public StyleAlreadyLoadedException(String message, Throwable cause) {
    super(message, cause);
  }
}
