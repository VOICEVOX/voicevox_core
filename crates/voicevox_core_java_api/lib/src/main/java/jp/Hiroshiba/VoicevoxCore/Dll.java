package jp.Hiroshiba.VoicevoxCore;

import ai.onnxruntime.OrtEnvironment;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

/** ライブラリを読み込むためだけのクラス。 */
abstract class Dll {
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
        osName = "windows";
        dllName = "voicevox_core_java_api.dll";
      } else if (rawOsName.startsWith("Mac")) {
        osName = "macos";
        dllName = "libvoicevox_core_java_api.dylib";
      } else if (rawOsName.startsWith("Linux")) {
        osName = "linux";
        dllName = "libvoicevox_core_java_api.so";
      } else {
        throw new RuntimeException("Unsupported OS: " + rawOsName);
      }
      if (rawOsArch.equals("x86")) {
        osArch = "x86";
      } else if (rawOsArch.equals("x86_64")) {
        osArch = "x64";
      } else if (rawOsArch.equals("amd64")) {
        osArch = "x64";
      } else if (rawOsArch.equals("aarch64")) {
        osArch = "arm64";
      } else {
        throw new RuntimeException("Unsupported OS architecture: " + rawOsArch);
      }

      String target = osName + "-" + osArch;
      // ONNX Runtime の DLL を読み込む。
      OrtEnvironment.getEnvironment();
      try (InputStream in = Dll.class.getResourceAsStream("/dll/" + target + "/" + dllName)) {
        if (in == null) {
          try {
            // フォールバック。開発用。
            System.loadLibrary("voicevox_core_java_api");
          } catch (UnsatisfiedLinkError e) {
            throw new RuntimeException("Failed to load Voicevox Core DLL for " + target, e);
          }
        } else {
          Path tempDir = Paths.get(System.getProperty("java.io.tmpdir"));
          Path dllPath = tempDir.resolve(dllName);
          dllPath.toFile().deleteOnExit();
          Files.copy(in, dllPath);

          System.load(dllPath.toAbsolutePath().toString());
        }
      } catch (Exception e) {
        throw new RuntimeException("Failed to load Voicevox Core DLL for " + target, e);
      }
    }
  }
}
