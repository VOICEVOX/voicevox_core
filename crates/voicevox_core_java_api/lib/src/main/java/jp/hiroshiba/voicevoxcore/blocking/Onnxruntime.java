package jp.hiroshiba.voicevoxcore.blocking;

import static jp.hiroshiba.voicevoxcore.GlobalInfo.SupportedDevices;

import com.google.gson.Gson;
import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;
import java.util.Optional;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/**
 * ONNX Runtime。
 *
 * <p>シングルトンであり、インスタンスは高々一つ。
 *
 * <pre>
 * Onnxruntime ort1 = Onnxruntime.loadOnce().exec();
 * Onnxruntime ort2 = Onnxruntime.get().get();
 * assert ort1 == ort2;
 * </pre>
 */
public class Onnxruntime {
  static {
    Dll.loadLibrary();
  }

  /** ONNX Runtimeのライブラリ名。 */
  public static final String LIB_NAME = "onnxruntime";

  /** 推奨されるONNX Runtimeのバージョン。 */
  public static final String LIB_VERSION = "1.17.3";

  /**
   * {@link LIB_NAME}と{@link LIB_VERSION}からなる動的ライブラリのファイル名。
   *
   * <p>WindowsとAndroidでは{@link LIB_UNVERSIONED_FILENAME}と同じ。
   */
  public static final String LIB_VERSIONED_FILENAME = rsLibVersionedFilename();

  /** {@link LIB_NAME}からなる動的ライブラリのファイル名。 */
  public static final String LIB_UNVERSIONED_FILENAME = rsLibUnversionedFilename();

  @Nullable private static Onnxruntime instance = null;

  /**
   * インスタンスが既に作られているならそれを得る。
   *
   * @return インスタンスがあるなら{@code Optional.of(…)}、そうでなければ{@code Optional.empty()}。
   */
  public static Optional<Onnxruntime> get() {
    synchronized (Onnxruntime.class) {
      return Optional.ofNullable(instance);
    }
  }

  /**
   * ONNX Runtimeをロードして初期化する。
   *
   * <p>一度成功したら、以後は引数を無視して同じインスタンスを返す。
   *
   * @return {@link LoadOnce}。
   */
  public static LoadOnce loadOnce() {
    return new LoadOnce();
  }

  private static native String rsLibName();

  private static native String rsLibVersion();

  private static native String rsLibVersionedFilename();

  private static native String rsLibUnversionedFilename();

  static {
    assert LIB_NAME.equals(rsLibName()) && LIB_VERSION.equals(rsLibVersion());
  }

  /** {@link #loadOnce}のビルダー。 */
  public static class LoadOnce {
    /**
     * ONNX Runtimeのファイル名（モジュール名）もしくはファイルパスを指定する。
     *
     * @param filename {@code dlopen}/<a
     *     href="https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw">{@code
     *     LoadLibraryExW}</a>の引数に使われる。デフォルトは{@link LIB_VERSIONED_FILENAME}。
     * @return このオブジェクト。
     */
    public LoadOnce filename(@Nonnull String filename) {
      this.filename = filename;
      return this;
    }

    /**
     * 実行する。
     *
     * @return {@link Onnxruntime}。
     */
    public Onnxruntime exec() {
      synchronized (Onnxruntime.class) {
        if (instance == null) {
          instance = new Onnxruntime(filename);
        }
      }
      return instance;
    }

    private LoadOnce() {}

    @Nonnull private String filename = LIB_VERSIONED_FILENAME;
  }

  private long handle;

  private Onnxruntime(@Nullable String filename) {
    rsNew(filename);
  }

  /**
   * このライブラリで利用可能なデバイスの情報を取得する。
   *
   * @return {@link SupportedDevices}。
   */
  public SupportedDevices supportedDevices() {
    Gson gson = new Gson();
    String supportedDevicesJson = rsSupportedDevices();
    SupportedDevices supportedDevices = gson.fromJson(supportedDevicesJson, SupportedDevices.class);
    if (supportedDevices == null) {
      throw new NullPointerException("supported_devices");
    }
    return supportedDevices;
  }

  private native void rsNew(@Nullable String filename);

  private native String rsSupportedDevices();
}
