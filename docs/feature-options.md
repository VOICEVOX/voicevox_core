## ONNX Runtimeのリンク方法のオプション

Rust API（`voicevox_core`）およびC API（`voicevox_core_c_api`）においては、ビルド時に
次のCargoフィーチャのうちどちらかを選択しなければなりません。
詳しくは[voicevox_core/Cargo.toml](../crates/voicevox_core/Cargo.toml)のコメントを参照して
下さい。Python APIやJava APIでは`onnxruntime-libloading`のみに限定しています。

- `onnxruntime-libloading`
- `onnxruntime-link-dylib`

```console
❯ cargo build --release -p voicevox_core_c_api --features onnxruntime-libloading
```

```console
❯ cargo build --release -p voicevox_core_c_api --features onnxruntime-link-dylib
```

C APIのリリースでは`dlopen`の利用が厳しいiOSでのみ`onnxruntime-link-dylib`で、その他
は`onnxruntime-libloading`でビルドしています。
