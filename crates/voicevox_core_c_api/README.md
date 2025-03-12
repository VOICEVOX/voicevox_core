# voicevox\_core\_c\_api

VOICEVOX COREのCバインディング。

## C APIのビルド

ビルドには [Rust](https://www.rust-lang.org/ja) ([Windows での Rust 開発環境構築手順はこちら](https://docs.microsoft.com/ja-jp/windows/dev-environment/rust/setup)) と [cmake](https://cmake.org/download/) が必要です。

```bash
# DLLをビルド
cargo build --release -p voicevox_core_c_api --features load-onnxruntime
```

DLL 用のヘッダファイルの雛形は [crates/voicevox_core_c_api/include/voicevox_core.h](https://github.com/VOICEVOX/voicevox_core/tree/main/crates/voicevox_core_c_api/include/voicevox_core.h) にあります。
詳しくは[feature-options.md](./docs/guide/user/feature-options.md)を参照してください。

```bash
# ヘッダファイルを加工し、マクロ`VOICEVOX_LOAD_ONNXRUNTIME`を宣言
sed 's:^//\(#define VOICEVOX_LOAD_ONNXRUNTIME\)$:\1:' \
  crates/voicevox_core_c_api/include/voicevox_core.h \
  > ./voicevox_core.h
```

## C APIライブラリのテスト

```bash
cargo test -p voicevox_core_c_api --features load-onnxruntime -- --include-ignored
```

## ヘッダファイルの更新

```bash
cargo xtask update-c-header
```
