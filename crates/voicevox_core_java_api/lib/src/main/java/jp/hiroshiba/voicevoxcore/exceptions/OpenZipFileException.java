package jp.hiroshiba.voicevoxcore.exceptions;

import java.io.IOException;

/** ZIPファイルを開くことに失敗した。 */
public final class OpenZipFileException extends IOException {
  public OpenZipFileException(String message) {
    super(message);
  }

  public OpenZipFileException(String message, Throwable cause) {
    super(message, cause);
  }
}
