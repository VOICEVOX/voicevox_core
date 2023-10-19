package jp.hiroshiba.voicevoxcore.exceptions;

/** AquesTalk風記法のテキストの解析に失敗した。 */
public class ParseKanaException extends IllegalArgumentException {
  public ParseKanaException(String message) {
    super(message);
  }

  public ParseKanaException(String message, Throwable cause) {
    super(message, cause);
  }
}
