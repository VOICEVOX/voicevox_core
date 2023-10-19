package jp.hiroshiba.voicevoxcore.exceptions;

/** コンテキストラベル出力に失敗した。 */
public class ExtractFullContextLabelException extends IllegalArgumentException {
  public ExtractFullContextLabelException(String message) {
    super(message);
  }

  public ExtractFullContextLabelException(String message, Throwable cause) {
    super(message, cause);
  }
}
