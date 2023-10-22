package jp.hiroshiba.voicevoxcore.exceptions;

/** open_jtalk辞書ファイルが読み込まれていない。 */
public class NotLoadedOpenjtalkDictException extends IllegalStateException {
  public NotLoadedOpenjtalkDictException(String message) {
    super(message);
  }

  public NotLoadedOpenjtalkDictException(String message, Throwable cause) {
    super(message, cause);
  }
}
