# voicevox_core_java_api

VOICEVOX CORE の Java バインディング。

## 環境構築

以下の環境が必要です：

- Rustup
- JDK 8

## ファイル構成

```yml
- README.md
- Cargo.toml # Rustプロジェクトとしてのマニフェストファイル。
- lib:
    - build.gradle # Gradle（Java）プロジェクトとしてのマニフェストファイル。
    - src:
        - main:
            - java:
                - jp/Hiroshiba/VoicevoxCore: # Javaのソースコード。
                    - Synthesizer.java
                    - ...
            - resources:
                - dll: # ライブラリ用のディレクトリ。詳細はDll.javaを参照。
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

- `LD_LIBRARY_PATH`などの環境変数に `[プロジェクトルート]/target/debug`（または`/release`） を追加するか、
- `lib/src/main/resources/dll/[target]/libvoicevox_core_java_api.so` を作成する

必要があります。

```console
❯ cargo build
❯ LD_LIBRARY_PATH=/home/user/voicevox_core/target/debug gradle build

# または
❯ cp ../../target/debug/libvoicevox_core_java_api.so lib/src/main/resources/dll/[target]/libvoicevox_core_java_api.so
❯ gradle build
```

## ビルド（リリース）

`cargo build --release` でビルドできます。

```console
❯ cargo build --release
❯ cp ../../target/release/libvoicevox_core_java_api.so lib/src/main/resources/dll/[target]/libvoicevox_core_java_api.so
```

## テスト

`gradle test` でテストできます。

```console
❯ gradle test
```

## ドキュメント

`gradle javadoc` でドキュメントを生成できます。

```console
❯ gradle javadoc
```

## DLL 読み込みについて

Android では、jniLibs から System.loadLibrary で読み込みます。

Android 以外では、src/main/resources/dll 内の適切な DLL を一時ディレクトリにコピーし、System.load で読み込みます。
DLL の名前は、

- Windows：voicevox_core_java_api.dll
- Linux：libvoicevox_core_java_api.so
- MacOS：libvoicevox_core_java_api.dylib

になります。
見付からなかった場合は、`System.loadLibrary` で読み込みます。これはデバッグ用です。
