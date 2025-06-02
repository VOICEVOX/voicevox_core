package jp.hiroshiba.voicevoxcore.exceptions;

/** 入力テキストの解析に失敗した。 */
public class AnalyzeTextException extends IllegalArgumentException {
  public AnalyzeTextException(String message) {
    super(message);
  }

  public AnalyzeTextException(String message, Throwable cause) {
    super(message, cause);
  }
}
