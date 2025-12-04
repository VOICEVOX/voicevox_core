package jp.hiroshiba.voicevoxcore.exceptions;

import java.io.IOException;

/** 推論に失敗した、もしくは推論結果が異常。 */
public class RunModelException extends IOException {
  public RunModelException(String message) {
    super(message);
  }

  public RunModelException(String message, Throwable cause) {
    super(message, cause);
  }
}
