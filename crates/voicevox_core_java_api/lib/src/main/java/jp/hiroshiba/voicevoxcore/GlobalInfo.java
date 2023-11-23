package jp.hiroshiba.voicevoxcore;

import com.google.gson.Gson;
import com.google.gson.annotations.Expose;
import com.google.gson.annotations.SerializedName;
import jakarta.annotation.Nonnull;

/** VOICEVOX CORE自体の情報。 */
public class GlobalInfo extends Dll {
  /**
   * ライブラリのバージョン。
   *
   * @return ライブラリのバージョン。
   */
  @Nonnull
  public static String getVersion() {
    return rsGetVersion();
  }

  /**
   * このライブラリで利用可能なデバイスの情報を取得する。
   *
   * @return {@link SupportedDevices}。
   */
  @Nonnull
  public static SupportedDevices getSupportedDevices() {
    Gson gson = new Gson();
    String supportedDevicesJson = rsGetSupportedDevicesJson();
    SupportedDevices supportedDevices = gson.fromJson(supportedDevicesJson, SupportedDevices.class);
    if (supportedDevices == null) {
      throw new NullPointerException("supported_devices");
    }
    return supportedDevices;
  }

  @Nonnull
  private static native String rsGetVersion();

  @Nonnull
  private static native String rsGetSupportedDevicesJson();

  /**
   * このライブラリで利用可能なデバイスの情報。
   *
   * <p>あくまで本ライブラリが対応しているデバイスの情報であることに注意。GPUが使える環境ではなかったとしても {@link #cuda} や {@link #dml} は {@code
   * true} を示しうる。
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
      this.cpu = false;
      this.cuda = false;
      this.dml = false;
    }
  }
}
