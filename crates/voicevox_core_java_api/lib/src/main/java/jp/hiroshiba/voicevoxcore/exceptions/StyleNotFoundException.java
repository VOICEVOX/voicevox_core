package jp.hiroshiba.voicevoxcore.exceptions;

/** スタイルIDに対するスタイルが見つからなかった。 */
public class StyleNotFoundException extends IndexOutOfBoundsException {
  public StyleNotFoundException(String message) {
    super(message);
  }

  public StyleNotFoundException(String message, Throwable cause)
      throws UnsupportedOperationException {
    throw new UnsupportedOperationException();
  }
}
