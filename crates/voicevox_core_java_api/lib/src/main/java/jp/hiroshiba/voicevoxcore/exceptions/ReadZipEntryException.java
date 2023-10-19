package jp.hiroshiba.voicevoxcore.exceptions;

import java.io.IOException;

/** ZIP内のファイルが読めなかった。 */
public class ReadZipEntryException extends IOException {
  public ReadZipEntryException(String message) {
    super(message);
  }

  public ReadZipEntryException(String message, Throwable cause) {
    super(message, cause);
  }
}
