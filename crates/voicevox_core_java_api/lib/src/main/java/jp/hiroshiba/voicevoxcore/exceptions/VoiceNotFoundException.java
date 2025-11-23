package jp.hiroshiba.voicevoxcore.exceptions;

/** 該当する声が見つからなかった。 */
public class VoiceNotFoundException extends IndexOutOfBoundsException {
  public VoiceNotFoundException(String message) {
    super(message);
  }

  public VoiceNotFoundException(String message, Throwable cause)
      throws UnsupportedOperationException {
    throw new UnsupportedOperationException();
  }
}
