package jp.hiroshiba.voicevoxcore.exceptions;

import java.io.IOException;

/** 推論に失敗した。 */
public class InferenceFailedException extends IOException {
  public InferenceFailedException(String message) {
    super(message);
  }

  public InferenceFailedException(String message, Throwable cause) {
    super(message, cause);
  }
}
