package jp.Hiroshiba.VoicevoxCore;

public class UserDict
    implements AutoCloseable {
  protected long internal;

  public UserDict() {
    rsNew();
  }

  public void close() {
    rsDrop();
  }

  private native void rsNew();

  private native void rsDrop();
}
