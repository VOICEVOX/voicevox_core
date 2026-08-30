package jp.hiroshiba.voicevoxcore.exceptions;

import java.io.IOException;

/** ユーザー辞書を読み込めなかった。 */
public final class LoadUserDictException extends IOException {
  public LoadUserDictException(String message) {
    super(message);
  }

  public LoadUserDictException(String message, Throwable cause) {
    super(message, cause);
  }
}
