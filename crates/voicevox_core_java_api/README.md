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

- `LD_LIBRARY_PATH`などの環境変数に `[プロジェクトルート]/target/debug`（または`/release`） を追加するか、
- `lib/src/main/resources/dll/[target]/libvoicevox_core_java_api.so` を作成する（`libvoicevox_core_java_api.so`はプラットフォームによって異なります、詳細は後述）。

必要があります。
また、ハードウェアアクセラレーションを有効にする時は`TARGET`環境変数を`cuda`または`directml`にし、Android 版をビルドする時は`TARGET`環境変数を`android`にしてください。

```console
❯ cargo build
❯ LD_LIBRARY_PATH=$(realpath ../../target/debug) ./gradlew test

# または
❯ cp ../../target/debug/libvoicevox_core_java_api.so lib/src/main/resources/dll/[target]/libvoicevox_core_java_api.so
❯ ./gradlew test
❯ TARGET=cuda ./gradlew test
```

## ビルド（リリース）

`cargo build --release` で Rust 側を、`./gradlew build` で Java 側をビルドできます。
パッケージ化する時は lib/src/main/resources/dll 内に dll をコピーしてください。
`TARGET`環境変数は開発時と同様に設定してください。

```console
❯ cargo build --release
❯ cp ../../target/release/libvoicevox_core_java_api.so lib/src/main/resources/dll/[target]/libvoicevox_core_java_api.so
❯ ./gradlew build
❯ TARGET=cuda ./gradlew build
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

Android 以外では、src/main/resources/dll 内の適切な DLL を一時ディレクトリにコピーし、System.load で読み込みます。
DLL の名前は、

- Windows：voicevox_core_java_api.dll
- Linux：libvoicevox_core_java_api.so
- macOS：libvoicevox_core_java_api.dylib

になります。
見付からなかった場合は、`System.loadLibrary` で読み込みます。これはデバッグ用です。
