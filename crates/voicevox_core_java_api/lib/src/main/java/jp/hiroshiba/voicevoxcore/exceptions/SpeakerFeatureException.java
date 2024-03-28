package jp.hiroshiba.voicevoxcore.exceptions;

public class SpeakerFeatureException extends UnsupportedOperationException {
  public SpeakerFeatureException(String message) {
    super(message);
  }

  public SpeakerFeatureException(String message, Throwable cause) {
    super(message, cause);
  }
}
