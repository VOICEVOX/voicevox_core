package jp.hiroshiba.voicevoxcore.internal;

import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.StandardCopyOption;

/** ライブラリを読み込むためだけ。 */
public class Dll {
  private static boolean loaded = false;

  private Dll() {
    throw new UnsupportedOperationException();
  }

  public static synchronized void loadLibrary() {
    if (loaded) return;

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

          // https://github.com/VOICEVOX/voicevox_core/issues/1042
          // Windowsだと`deleteOnExit`が機能せずに残り続けるらしいため、`REPLACE_EXISTING`でコピー。
          Files.copy(in, dllPath, StandardCopyOption.REPLACE_EXISTING);

          System.load(dllPath.toAbsolutePath().toString());
        }
      } catch (Exception e) {
        // FIXME: `tempDir`の削除
        throw new RuntimeException("Failed to load Voicevox Core DLL for " + target, e);
      }
    }

    new LoggerInitializer().initLogger();

    loaded = true;
  }

  static class LoggerInitializer {
    native void initLogger();
  }
}
