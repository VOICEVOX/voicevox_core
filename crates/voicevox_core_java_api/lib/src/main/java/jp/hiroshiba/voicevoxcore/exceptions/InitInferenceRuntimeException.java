package jp.hiroshiba.voicevoxcore.exceptions;

import java.io.IOException;

/** 推論ライブラリのロードまたは初期化ができなかった。 */
public class InitInferenceRuntimeException extends IOException {
  public InitInferenceRuntimeException(String message) {
    super(message);
  }

  public InitInferenceRuntimeException(String message, Throwable cause) {
    super(message, cause);
  }
}
