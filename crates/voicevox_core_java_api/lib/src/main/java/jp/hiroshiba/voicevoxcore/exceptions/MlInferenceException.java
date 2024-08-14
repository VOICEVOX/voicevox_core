package jp.hiroshiba.voicevoxcore.exceptions;

import java.io.IOException;

/** 推論に失敗した。 */
public class MlInferenceException extends IOException {
  public MlInferenceException(String message) {
    super(message);
  }

  public MlInferenceException(String message, Throwable cause) {
    super(message, cause);
  }
}
