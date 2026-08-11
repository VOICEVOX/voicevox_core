package jp.hiroshiba.voicevoxcore.blocking;

import jakarta.annotation.Nonnull;
import jakarta.annotation.Nullable;
import java.util.Optional;
import jp.hiroshiba.voicevoxcore.SupportedDevices;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/**
 * ONNX Runtime。
 *
 * <p>シングルトンであり、インスタンスは高々一つ。
 *
 * <pre>
 * Onnxruntime ort1 = Onnxruntime.loadOnce().perform();
 * Onnxruntime ort2 = Onnxruntime.get().get();
 * assert ort1 == ort2;
 * </pre>
 */
public class Onnxruntime {
  static {
    Dll.loadLibrary();
  }

  /** 必要なONNX Runtime 1.xの最小マイナーバージョン。 */
  public static final int LIB_MIN_REQUIRED_VERSION = 17;

  /** サポートされるONNX Runtime 1.xの最大マイナーバージョン。 */
  public static final int LIB_MAX_SUPPORTED_VERSION = 17;

  /** 推奨されるONNX Runtimeのライブラリ名。 */
  public static final String LIB_RECOMMENDED_NAME = "voicevox_onnxruntime";

  /** 推奨されるONNX Runtimeのバージョン。 */
  public static final String LIB_RECOMMENDED_VERSION = "1.17.3";

  /**
   * {@link LIB_RECOMMENDED_NAME}と{@link LIB_RECOMMENDED_VERSION}からなる動的ライブラリのファイル名。
   *
   * <p>WindowsとAndroidでは{@link LIB_RECOMMENDED_UNVERSIONED_FILENAME}と同じ。
   */
  public static final String LIB_RECOMMENDED_VERSIONED_FILENAME =
      rsLibRecommendedVersionedFilename();

  /** {@link LIB_RECOMMENDED_NAME}からなる動的ライブラリのファイル名。 */
  public static final String LIB_RECOMMENDED_UNVERSIONED_FILENAME =
      rsLibRecommendedUnversionedFilename();

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
   * <p>対象のONNX Runtimeはバージョン<code>1.{@link #LIB_MIN_REQUIRED_VERSION}</code>以降のものでなければならない。バージョン
   * <code>1.{@link #LIB_MAX_SUPPORTED_VERSION}</code>よりも新しいONNX Runtimeに対しては警告を出す。
   *
   * <p>一度成功したら、以後は引数を無視して同じインスタンスを返す。
   *
   * @return {@link LoadOnce}。
   */
  public static LoadOnce loadOnce() {
    return new LoadOnce();
  }

  private static native int rsLibMinRequiredVersion();

  private static native int rsLibMaxSupportedVersion();

  private static native String rsLibRecommendedName();

  private static native String rsLibRecommendedVersion();

  private static native String rsLibRecommendedVersionedFilename();

  private static native String rsLibRecommendedUnversionedFilename();

  static {
    assert LIB_MIN_REQUIRED_VERSION == rsLibMinRequiredVersion()
        && LIB_MAX_SUPPORTED_VERSION == rsLibMaxSupportedVersion()
        && LIB_RECOMMENDED_NAME.equals(rsLibRecommendedName())
        && LIB_RECOMMENDED_VERSION.equals(rsLibRecommendedVersion());
  }

  /** {@link #loadOnce}のビルダー。 */
  public static class LoadOnce {
    /**
     * ONNX Runtimeのファイル名（モジュール名）もしくはファイルパスを指定する。
     *
     * @param filename {@code dlopen}/<a
     *     href="https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw">{@code
     *     LoadLibraryExW}</a>の引数に使われる。デフォルトは{@link LIB_RECOMMENDED_VERSIONED_FILENAME}。
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
    public Onnxruntime perform() {
      synchronized (Onnxruntime.class) {
        if (instance == null) {
          instance = new Onnxruntime(filename);
        }
      }
      return instance;
    }

    private LoadOnce() {}

    @Nonnull private String filename = LIB_RECOMMENDED_VERSIONED_FILENAME;
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
    return rsSupportedDevices();
  }

  private native void rsNew(@Nullable String filename);

  private native SupportedDevices rsSupportedDevices();
}
