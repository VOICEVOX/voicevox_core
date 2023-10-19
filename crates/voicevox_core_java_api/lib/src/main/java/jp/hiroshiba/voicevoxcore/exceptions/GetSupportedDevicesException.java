package jp.hiroshiba.voicevoxcore.exceptions;

import java.io.IOException;

/** サポートされているデバイス情報取得に失敗した。 */
public class GetSupportedDevicesException extends IOException {
  public GetSupportedDevicesException(String message) {
    super(message);
  }

  public GetSupportedDevicesException(String message, Throwable cause) {
    super(message, cause);
  }
}
