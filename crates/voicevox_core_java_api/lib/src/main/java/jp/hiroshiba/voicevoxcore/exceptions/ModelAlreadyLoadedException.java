package jp.hiroshiba.voicevoxcore.exceptions;

/** すでに読み込まれている音声モデルを読み込もうとした。 */
public class ModelAlreadyLoadedException extends IllegalStateException {
  public ModelAlreadyLoadedException(String message) {
    super(message);
  }

  public ModelAlreadyLoadedException(String message, Throwable cause) {
    super(message, cause);
  }
}
