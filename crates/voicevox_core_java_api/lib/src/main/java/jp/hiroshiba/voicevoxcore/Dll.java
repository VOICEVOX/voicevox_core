package jp.hiroshiba.voicevoxcore;

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
      String osName, osArch, vvDllName, ortDllName;
      String[] ortOptionalDllNames;
      if (rawOsName.startsWith("Win")) {
        osName = "windows";
        vvDllName = "voicevox_core_java_api.dll";
        ortDllName = "onnxruntime.dll";
        ortOptionalDllNames =
            new String[] {"onnxruntime_providers_shared.dll", "onnxruntime_providers_cuda.dll"};
      } else if (rawOsName.startsWith("Mac")) {
        osName = "macos";
        vvDllName = "libvoicevox_core_java_api.dylib";
        ortDllName = "libonnxruntime.1.14.0.dylib";
        ortOptionalDllNames = new String[] {};
      } else if (rawOsName.startsWith("Linux")) {
        osName = "linux";
        vvDllName = "libvoicevox_core_java_api.so";
        ortDllName = "libonnxruntime.so.1.14.0";
        ortOptionalDllNames =
            new String[] {"libonnxruntime_providers_shared.so", "libonnxruntime_providers_cuda.so"};
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
      loadDll(target, ortDllName, "onnxruntime");
      for (String dllName : ortOptionalDllNames) {
        loadDll(target, dllName);
      }
      loadDll(target, vvDllName, "voicevox_core_java_api");
    }
  }

  private static void loadDll(String target, String dllName) {
    loadDll(target, dllName, null);
  }

  private static void loadDll(String target, String dllName, String fallbackDllName) {
    String resourceRoot = "/dll/" + target + "/";
    try (InputStream in = Dll.class.getResourceAsStream(resourceRoot + dllName)) {
      if (in == null) {
        if (fallbackDllName == null) {
          return;
        }
        try {
          System.loadLibrary(fallbackDllName);
        } catch (UnsatisfiedLinkError e) {
          throw new RuntimeException("Failed to load " + dllName + " for " + target, e);
        }
      } else {
        Path tempDir = Paths.get(System.getProperty("java.io.tmpdir"));
        Path dllPath = tempDir.resolve(dllName);
        dllPath.toFile().deleteOnExit();
        Files.copy(in, dllPath);

        System.load(dllPath.toAbsolutePath().toString());
      }
    } catch (Exception e) {
      throw new RuntimeException("Failed to load " + dllName + " for " + target, e);
    }
  }
}
