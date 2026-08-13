package jp.hiroshiba.voicevoxcore.exceptions;

import java.io.IOException;

/** モデルの形式が不正。 */
public final class InvalidModelFormatException extends IOException {
  public InvalidModelFormatException(String message) {
    super(message);
  }

  public InvalidModelFormatException(String message, Throwable cause) {
    super(message, cause);
  }
}
