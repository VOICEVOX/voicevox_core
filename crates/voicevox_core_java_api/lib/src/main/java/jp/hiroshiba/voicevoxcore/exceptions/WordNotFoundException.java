package jp.hiroshiba.voicevoxcore.exceptions;

/** ユーザー辞書に単語が見つからなかった。 */
public final class WordNotFoundException extends IndexOutOfBoundsException {
  public WordNotFoundException(String message) {
    super(message);
  }

  public WordNotFoundException(String message, Throwable cause)
      throws UnsupportedOperationException {
    throw new UnsupportedOperationException();
  }
}
