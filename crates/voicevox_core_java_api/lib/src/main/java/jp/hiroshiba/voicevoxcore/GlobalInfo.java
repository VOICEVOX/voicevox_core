package jp.hiroshiba.voicevoxcore;

import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;
import jp.hiroshiba.voicevoxcore.internal.Dll;

/** VOICEVOX CORE自体の情報。 */
public class GlobalInfo {
  static {
    Dll.loadLibrary();
  }

  /**
   * ライブラリのバージョン。
   *
   * @return ライブラリのバージョン。
   */
  @Nonnull
  public static String getVersion() {
    return rsGetVersion();
  }

  @Nonnull
  private static native String rsGetVersion();

  // FIXME: dead code
  @Nonnull
  private static native String rsGetSupportedDevicesJson();

  // FIXME: `Onnxruntime`に移すか、独立させる
  /**
   * ONNX Runtime利用可能なデバイスの情報。
   *
   * <p>あくまでONNX Runtimeが対応しているデバイスの情報であることに注意。GPUが使える環境ではなかったとしても {@link #cuda} や {@link #dml} は
   * {@code true} を示しうる。
   *
   * <p>現在この型はGSONに対応しているが、将来的には <a href="https://github.com/VOICEVOX/voicevox_core/issues/984"
   * target="_blank">Jacksonに切り替わる予定</a> 。
   *
   * <p>{@code Gson#fromJson} でJSONから変換することはできない。その試みは {@link UnsupportedOperationException} となる。
   */
  public static class SupportedDevices {
    /**
     * CPUが利用可能。
     *
     * <p>常に <code>true</code> 。
     */
    @SerializedName("cpu")
    @Expose
    @Nonnull
    public final boolean cpu;

    /**
     * CUDAが利用可能。
     *
     * <p>ONNX Runtimeの <a href=
     * "https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html"
     * target="_blank">CUDAExecutionProvider</a>に対応する。 必要な環境についてはそちらを参照。
     */
    @SerializedName("cuda")
    @Expose
    @Nonnull
    public final boolean cuda;

    /**
     * DirectMLが利用可能。
     *
     * <p>ONNX Runtimeの <a href=
     * "https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html"
     * target="_blank">DmlExecutionProvider</a>に対応する。 必要な環境についてはそちらを参照。
     */
    @SerializedName("dml")
    @Expose
    @Nonnull
    public final boolean dml;

    private SupportedDevices() {
      throw new UnsupportedOperationException("You cannot deserialize `SupportedDevices`");
    }

    /** accessed only via JNI */
    private SupportedDevices(boolean cpu, boolean cuda, boolean dml) {
      this.cpu = cpu;
      this.cuda = cuda;
      this.dml = dml;
    }
  }
}
