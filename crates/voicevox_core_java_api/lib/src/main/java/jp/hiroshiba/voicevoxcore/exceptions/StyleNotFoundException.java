package jp.hiroshiba.voicevoxcore.exceptions;

/** スタイルIDに対するスタイルが見つからなかった。 */
public final class StyleNotFoundException extends IndexOutOfBoundsException {
  public StyleNotFoundException(String message) {
    super(message);
  }

  public StyleNotFoundException(String message, Throwable cause)
      throws UnsupportedOperationException {
    throw new UnsupportedOperationException();
  }
}
