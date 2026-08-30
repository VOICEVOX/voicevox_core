package jp.hiroshiba.voicevoxcore.exceptions;

import java.io.IOException;

/** ユーザー辞書を書き込めなかった。 */
public final class SaveUserDictException extends IOException {
  public SaveUserDictException(String message) {
    super(message);
  }

  public SaveUserDictException(String message, Throwable cause) {
    super(message, cause);
  }
}
