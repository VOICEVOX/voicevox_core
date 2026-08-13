package jp.hiroshiba.voicevoxcore.exceptions;

import java.io.IOException;

/** 無効なモデルデータ。 */
public final class InvalidModelDataException extends IOException {
  public InvalidModelDataException(String message) {
    super(message);
  }

  public InvalidModelDataException(String message, Throwable cause) {
    super(message, cause);
  }
}
