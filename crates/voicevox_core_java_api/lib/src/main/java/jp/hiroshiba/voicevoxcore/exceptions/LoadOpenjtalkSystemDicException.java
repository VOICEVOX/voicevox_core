package jp.hiroshiba.voicevoxcore.exceptions;

/** Open JTalkのシステム辞書を読むことができなかった。 */
public class LoadOpenjtalkSystemDicException extends IllegalStateException {
  public LoadOpenjtalkSystemDicException(String message) {
    super(message);
  }

  public LoadOpenjtalkSystemDicException(String message, Throwable cause) {
    super(message, cause);
  }
}
