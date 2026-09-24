# WebAssembly (browser) example — experimental

VOICEVOX Core を `wasm32-unknown-emscripten` 向けにビルドし、ブラウザの Web Worker 上で音声合成するサンプルです。
Worker で「これはテストです」を合成し、生成された WAV をメインスレッドに転送して `<audio>` で再生します。

## 構成

| ファイル | 役割 |
| --- | --- |
| `index.html` | ログ表示と音声プレイヤー |
| `main.js` | Worker を起動し、受け取った WAV を検証して再生 |
| `worker.js` | `sample.vvm` を Emscripten FS に書き込み、Core で合成 |
| `wasm_browser_api_example.{js,wasm}` | ビルド成果物（手順 3 でコピー） |
| `sample.vvm` | 音声モデル（手順 4 で作成） |

## 必要なもの

- Rust 1.96.0（`wasm32-unknown-emscripten` ターゲット）
- [Emscripten SDK](https://emscripten.org/docs/getting_started/downloads.html) 4.0.8
- WebAssembly 向けにビルドした ONNX Runtime（`libonnxruntime_webassembly.a`）

以下では次の変数を使います。

- `EMSDK_PATH`: emsdk のディレクトリ
- `VOICEVOX_CORE_PATH`: voicevox_core リポジトリのルート
- `ONNXRUNTIME_DOWNLOAD_DIR`: ONNX Runtime の静的ライブラリがあるディレクトリ

## 手順

### 1. ツールチェーンの準備

```sh
rustup toolchain install 1.96.0 --profile minimal --target wasm32-unknown-emscripten

cd "${EMSDK_PATH}"
./emsdk install 4.0.8
./emsdk activate 4.0.8
source ./emsdk_env.sh
```

### 2. ビルド

```sh
cd "${VOICEVOX_CORE_PATH}"

export CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_LINKER=emcc
export CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_RUSTFLAGS="\
  -C target-feature=+simd128 \
  -C link-arg=-sALLOW_MEMORY_GROWTH=1 \
  -C link-arg=-sSTACK_SIZE=8388608 \
  -C link-arg=-sEXPORTED_RUNTIME_METHODS=FS,HEAPU8 \
  -C link-arg=-sEXPORTED_FUNCTIONS=_main,_voicevox_browser_synthesize,_voicevox_browser_wav_pointer,_voicevox_browser_wav_length \
  -L native=${ONNXRUNTIME_DOWNLOAD_DIR}"

cargo +1.96.0 build \
  --package voicevox_core_wasm \
  --bin wasm_browser_api_example \
  --features link-onnxruntime \
  --target wasm32-unknown-emscripten \
  --profile c-api
```

### 3. 成果物のコピー

```sh
cp target/wasm32-unknown-emscripten/c-api/wasm_browser_api_example.{js,wasm} \
  example/wasm-experimental/
```

### 4. 音声モデルの用意

`model/sample.vvm` はディレクトリなので、中身を zip にまとめて配置します。

```sh
(cd model/sample.vvm && zip -r ../../example/wasm-experimental/sample.vvm .)
```

### 5. 実行

Worker と `fetch` を使うため、`file://` ではなく HTTP サーバー経由で開きます。

```sh
cd example/wasm-experimental
python3 -m http.server 8000
```

<http://localhost:8000/> を開くと合成が始まり、完了するとプレイヤーが表示されます。
ブラウザの自動再生制限でブロックされた場合は、再生ボタンを押してください。

`body` の `data-status` 属性は `running` → `passed` / `failed` と変化するため、自動テストの判定に使えます。
