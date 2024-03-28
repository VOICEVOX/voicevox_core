package jp.hiroshiba.voicevoxcore.exceptions;

import java.io.IOException;

/** モデルデータが見つからなかった。 */
public class MissingModelDataException extends IOException {
  public MissingModelDataException(String message) {
    super(message);
  }

  public MissingModelDataException(String message, Throwable cause) {
    super(message, cause);
  }
}
