package jp.Hiroshiba.VoicevoxCore;

import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

import ai.onnxruntime.OrtEnvironment;

/** ライブラリを読み込むためだけのクラス。 */
abstract class Dll {
  // DLL を読み込む。
  // src/main/resources/dll/[target] 以下に DLL を配置する。
  // targetには以下のいずれかが入る：
  // - win-x64
  // - mac-x64
  // - mac-arm64
  // - linux-x64
  // - linux-arm64
  static {
    String runtimeName = System.getProperty("java.runtime.name");
    if (runtimeName.equals("Android Runtime")) {
      // Android ではjniLibsから読み込む。
      System.loadLibrary("voicevox_core_java_api");
    } else {
      String rawOsName = System.getProperty("os.name");
      String rawOsArch = System.getProperty("os.arch");
      String osName, osArch, dllName;
      if (rawOsName.startsWith("Win")) {
        osName = "win";
        dllName = "voicevox_core_java_api.dll";
      } else if (rawOsName.startsWith("Mac")) {
        osName = "mac";
        dllName = "libvoicevox_core_java_api.dylib";
      } else if (rawOsName.startsWith("Linux")) {
        osName = "linux";
        dllName = "libvoicevox_core_java_api.so";
      } else {
        throw new RuntimeException("Unsupported OS: " + rawOsName);
      }
      if (rawOsArch.equals("x86_64")) {
        osArch = "x64";
      } else if (rawOsArch.equals("amd64")) {
        osArch = "x64";
      } else if (rawOsArch.equals("aarch64")) {
        osArch = "arm64";
      } else {
        throw new RuntimeException("Unsupported OS architecture: " + rawOsArch);
      }

      // ONNX Runtime の DLL を読み込む。
      OrtEnvironment.getEnvironment();
      try (InputStream in = Dll.class.getResourceAsStream("/dll/" + osName + "-" + osArch + "/" + dllName)) {
        if (in == null) {
          try {
            // フォールバック。開発用。
            System.loadLibrary("voicevox_core_java_api");
          } catch (UnsatisfiedLinkError e) {
            throw new RuntimeException("Failed to load Voicevox Core DLL", e);
          }
        } else {
          Path tempDir = Paths.get(System.getProperty("java.io.tmpdir"));
          tempDir = tempDir.resolve("voicevox_core_java_api");
          tempDir.toFile().mkdirs();
          Path dllPath = tempDir.resolve(dllName);
          dllPath.toFile().deleteOnExit();
          Files.copy(in, dllPath);

          System.load(dllPath.toAbsolutePath().toString());
        }
      } catch (Exception e) {
        throw new RuntimeException("Failed to load Voicevox Core DLL", e);
      }
    }
  }
}
