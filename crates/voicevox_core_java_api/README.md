# voicevox_core_java_api

VOICEVOX CORE の Java バインディング。

## 環境構築

以下の環境が必要です。

- Rustup
- Java 8

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
            - jniLibs:
                - x86_64:
                    - ...
- src: # Rustのソースコード。jni-rsを使ってJavaとのバインディングを行う。
    - lib.rs
    - ...
```

## ビルド（開発）

バインディングは `cargo build` でビルドできます。
Java プロジェクトを動かすには、`LD_LIBRARY_PATH`（等の環境変数）に `[プロジェクトルート]/target/debug(release)` を追加する必要があります。

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
