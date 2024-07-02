## ONNX Runtimeのリンク方法のオプション

Rust API（`voicevox_core`）およびC API（`voicevox_core_c_api`）においては、ビルド時に
次のCargoフィーチャのうちどちらかを選択しなければなりません。
詳しくは[voicevox_core/Cargo.toml](../crates/voicevox_core/Cargo.toml)のコメントを参照して
下さい。Python APIやJava APIでは`load-onnxruntime`のみに限定しています。

- `load-onnxruntime`
- `link-onnxruntime`

```console
❯ cargo build --release -p voicevox_core_c_api --features load-onnxruntime
❯ sed 's:^//\(#define VOICEVOX_LOAD_ONNXRUNTIME\)$:\1:' \
  crates/voicevox_core_c_api/include/voicevox_core.h \
  > ./voicevox_core.h
```

```console
❯ cargo build --release -p voicevox_core_c_api --features link-onnxruntime
❯ sed 's:^//\(#define VOICEVOX_LINK_ONNXRUNTIME\)$:\1:' \
  crates/voicevox_core_c_api/include/voicevox_core.h \
  > ./voicevox_core.h
```

C APIのリリースでは`dlopen`の利用が厳しいiOSでのみ`link-onnxruntime`で、その他は`load-onnxruntime`で
ビルドしています。
