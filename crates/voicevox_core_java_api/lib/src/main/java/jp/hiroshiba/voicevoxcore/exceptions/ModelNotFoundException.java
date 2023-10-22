package jp.hiroshiba.voicevoxcore.exceptions;

/** 音声モデルIDに対する音声モデルが見つからなかった。 */
public class ModelNotFoundException extends IndexOutOfBoundsException {
  public ModelNotFoundException(String message) {
    super(message);
  }

  public ModelNotFoundException(String message, Throwable cause)
      throws UnsupportedOperationException {
    throw new UnsupportedOperationException();
  }
}
