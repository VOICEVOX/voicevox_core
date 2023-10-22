package jp.hiroshiba.voicevoxcore.exceptions;

/** GPUモードがサポートされていない。 */
public class GpuSupportException extends RuntimeException {
  public GpuSupportException(String message) {
    super(message);
  }

  public GpuSupportException(String message, Throwable cause) {
    super(message, cause);
  }
}
