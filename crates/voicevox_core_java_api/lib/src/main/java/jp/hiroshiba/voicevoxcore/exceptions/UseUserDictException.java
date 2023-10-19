package jp.hiroshiba.voicevoxcore.exceptions;

/** OpenJTalkのユーザー辞書の設定に失敗した。 */
public class UseUserDictException extends RuntimeException {
  public UseUserDictException(String message) {
    super(message);
  }

  public UseUserDictException(String message, Throwable cause) {
    super(message, cause);
  }
}
