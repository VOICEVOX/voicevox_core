package jp.hiroshiba.voicevoxcore.exceptions;

public class AmbiguousVoiceException extends RuntimeException {
  public AmbiguousVoiceException(String message) {
    super(message);
  }

  public AmbiguousVoiceException(String message, Throwable cause)
      throws UnsupportedOperationException {
    throw new UnsupportedOperationException();
  }
}
