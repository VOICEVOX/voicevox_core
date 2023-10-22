package jp.hiroshiba.voicevoxcore.exceptions;

/** ユーザー辞書の単語のバリデーションに失敗した。 */
public class InvalidWordException extends IllegalArgumentException {
  public InvalidWordException(String message) {
    super(message);
  }

  public InvalidWordException(String message, Throwable cause) {
    super(message, cause);
  }
}
