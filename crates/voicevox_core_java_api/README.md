# voicevox_core_java_api

VOICEVOX CORE の Java バインディング。

## 環境構築

以下の環境が必要です：

- Rustup
- JDK 11

## ファイル構成

```yml
- README.md
- Cargo.toml # Rustプロジェクトとしてのマニフェストファイル。
- lib:
    - build.gradle # Gradle（Java）プロジェクトとしてのマニフェストファイル。
    - src:
        - main:
            - java:
                - jp/hiroshiba/voicevoxcore: # Javaのソースコード。
                    - Synthesizer.java
                    - ...
            - resources:
                - dll: # ライブラリ用のディレクトリ。詳細は後述。
                    - windows-x64:
                        - voicevox_core.dll
                    - ...
                - jniLibs: # Android用のディレクトリ。
                    - x86_64:
                        - ...
- src: # Rustのソースコード。jni-rsを使ってJavaとのバインディングを行う。
    - lib.rs
    - ...
```

## ビルド（開発）

バインディングは `cargo build` でビルドできます。
Java プロジェクトを動かすには、

- `LD_LIBRARY_PATH`などの環境変数に `[プロジェクトルート]/target/debug`（または`/release`） や onnxruntime の DLL があるディレクトリを追加するか、
- `lib/src/main/resources/dll/[target]/`内に onnxruntime と voicevox_core_java_api の DLL をコピーする

必要があります。

```console
❯ cargo build
❯ export LD_LIBRARY_PATH="$(realpath ../../target/debug):$LD_LIBRARY_PATH"
❯ export LD_LIBRARY_PATH="/path/to/onnxruntime/lib:$LD_LIBRARY_PATH"
❯ ./gradlew build

# または
❯ cp ../../target/debug/libvoicevox_core_java_api.so lib/src/main/resources/dll/[target]/libvoicevox_core_java_api.so
❯ cp /path/to/onnxruntime/lib/libonnxruntime.so.1.14.0 lib/src/main/resources/dll/[target]/libonnxruntime.so.1.14.0
❯ ./gradlew build
```

## ビルド（リリース）

`cargo build --release` で Rust 側を、`./gradlew build` で Java 側をビルドできます。
パッケージ化する時は lib/src/main/resources/dll 内に DLL をコピーしてください。

```console
❯ cargo build --release
❯ cp ../../target/release/libvoicevox_core_java_api.so lib/src/main/resources/dll/[target]/libvoicevox_core_java_api.so
❯ cp /path/to/onnxruntime/lib/libonnxruntime.so.1.14.0 lib/src/main/resources/dll/[target]/libonnxruntime.so.1.14.0
❯ ./gradlew build
```

## テスト

`./gradlew test` でテストできます。

```console
❯ ./gradlew test
```

## ドキュメント

`./gradlew javadoc` でドキュメントを生成できます。

```console
❯ ./gradlew javadoc
```

## DLL 読み込みについて

Android では、jniLibs から System.loadLibrary で読み込みます。

Android 以外では、src/main/resources/dll 内の DLL を一時ディレクトリにコピーし、System.load で読み込みます。
見付からなかった場合は、`System.loadLibrary` で読み込みます。これはデバッグ用です。
