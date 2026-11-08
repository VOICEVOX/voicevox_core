# WebAssembly (browser) example — experimental

VOICEVOX Core を `wasm32-unknown-emscripten` 向けにビルドし、ブラウザの Web Worker 上で音声合成するサンプルです。
Worker で Core を初期化した後、ページのテキストボックスに入力した任意の文字列を合成し、生成された WAV をメインスレッドに転送して `<audio>` で再生します。

## 構成

| ファイル | 役割 |
| --- | --- |
| `index.html` | ログ表示と音声プレイヤー |
| `main.js` | Worker を起動し、入力されたテキストを送信、受け取った WAV を検証して再生 |
| `worker.js` | (module Worker) 辞書と `sample.vvm` を Emscripten FS に書き込んで Core を初期化し、メッセージごとに AudioQuery 生成→合成 |
| `wasm_browser_api_example.{js,wasm}` | ビルド成果物（手順 3 でコピー）。`createVoicevoxCore` を default export する ES module |
| `sample.vvm` | 音声モデル（手順 4 で作成） |
| `open_jtalk_dic_utf_8-1.11/` | Open JTalk 辞書（手順 4 で配置） |

## API

`crates/voicevox_core_wasm/src/main.rs` が `voicevox_wasm_*` 関数を export します。
文字列引数は NUL 終端 UTF-8、ファイルは Emscripten FS 上のパスで渡します。
各関数は成功で `0`、失敗で `1` を返し、結果（JSON / WAV）または失敗時のエラーメッセージが結果バッファに入ります。
結果バッファは `voicevox_wasm_result_pointer` / `voicevox_wasm_result_length` で `HEAPU8` から読み出します。

| 関数 | 結果 |
| --- | --- |
| `voicevox_wasm_initialize(dict_dir)` | なし |
| `voicevox_wasm_load_voice_model(vvm_path)` | そのVVMのメタ情報 (JSON) |
| `voicevox_wasm_metas()` | 読み込み済み全VVMのメタ情報 (JSON) |
| `voicevox_wasm_is_loaded_style(style_id)` | 戻り値が `1`/`0`（結果バッファは変更しない） |
| `voicevox_wasm_create_audio_query(text, style_id)` | AudioQuery (JSON) |
| `voicevox_wasm_create_accent_phrases(text, style_id)` | AccentPhrase[] (JSON) |
| `voicevox_wasm_create_accent_phrases_from_kana(kana, style_id)` | AccentPhrase[] (JSON) |
| `voicevox_wasm_replace_mora_data(accent_phrases_json, style_id)` | AccentPhrase[] (JSON) |
| `voicevox_wasm_synthesis(audio_query_json, style_id, enable_interrogative_upspeak)` | WAV |
| `voicevox_wasm_tts(text, style_id)` | WAV |

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
  -C link-arg=-sINITIAL_MEMORY=1073741824 \
  -C link-arg=-sSTACK_SIZE=8388608 \
  -C link-arg=-sMODULARIZE=1 \
  -C link-arg=-sEXPORT_ES6=1 \
  -C link-arg=-sEXPORT_NAME=createVoicevoxCore \
  -C link-arg=-sENVIRONMENT=web,worker \
  -C link-arg=-sEXPORTED_RUNTIME_METHODS=FS,HEAPU8,ccall \
  -C link-arg=-sEXPORTED_FUNCTIONS=_main,_voicevox_wasm_initialize,_voicevox_wasm_load_voice_model,_voicevox_wasm_metas,_voicevox_wasm_is_loaded_style,_voicevox_wasm_create_audio_query,_voicevox_wasm_create_accent_phrases,_voicevox_wasm_create_accent_phrases_from_kana,_voicevox_wasm_replace_mora_data,_voicevox_wasm_synthesis,_voicevox_wasm_tts,_voicevox_wasm_result_pointer,_voicevox_wasm_result_length \
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

### 4. 音声モデルと辞書の用意

`model/sample.vvm` はディレクトリなので、中身を zip にまとめて配置します。

```sh
(cd model/sample.vvm && zip -r ../../example/wasm-experimental/sample.vvm .)
```

Open JTalk 辞書を `open_jtalk_dic_utf_8-1.11` ディレクトリごと配置します。
Worker は辞書の各ファイルを個別に `fetch` し、Emscripten FS の `/open_jtalk_dic_utf_8-1.11` に書き込みます。

### 5. 実行

Worker と `fetch` を使うため、`file://` ではなく HTTP サーバー経由で開きます。

```sh
cd example/wasm-experimental
python3 -m http.server 8000
```

<http://localhost:8000/> を開くと辞書とモデルの読み込みが始まり、完了すると「合成」ボタンが押せるようになります。
テキストと Style ID を指定して「合成」を押すと、完了後にプレイヤーが表示されます。
ブラウザの自動再生制限でブロックされた場合は、再生ボタンを押してください。

`body` の `data-status` 属性は `loading` → `ready` → (合成ごとに) `running` → `passed` / `failed` と変化するため、自動テストの判定に使えます。
