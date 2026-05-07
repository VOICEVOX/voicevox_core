# VOICEVOX CLI Example

`voicevox_core` を使用してテキストを音声に合成するコマンドラインツール（CLI）のサンプル実装です。

## 準備

このサンプルを実行するには、ONNX Runtime ライブラリ、音声モデル、Open JTalk 辞書が必要です。
これらは `downloader` ツールを使用して一括でダウンロードできます。

リポジトリのルートディレクトリで以下のコマンドを実行してください。

```bash
cargo run -p downloader
```

このコマンドを実行すると、デフォルトでプロジェクトルートの `voicevox_core/` ディレクトリに必要なリソースがダウンロード・展開されます。
本 CLI サンプルは、デフォルトでこのディレクトリ内のリソースを参照するように構成されています。

## ビルド

リポジトリのサンプルディレクトリで以下のコマンドを実行します。

```bash
cd example/cli/
cargo build
```

## 使い方

```bash
cd example/cli/
cargo run -- --tts "こんにちは、音声合成のテストです。" --output output.wav
```

### コマンドライン引数

| 引数 | 説明 | デフォルト値 |
| :--- | :--- | :--- |
| `--tts` | 合成するテキスト（必須） | - |
| `--output` | 出力するWAVファイルのパス | `./output.wav` |
| `--character` | キャラクター名 | `ずんだもん` |
| `--style` | スタイル名 | `ノーマル` |
| `--model` | VVMファイルのパス | `./voicevox_core/models/vvms/0.vvm` |
| `--onnxruntime` | ONNX Runtime ライブラリのパス | `./voicevox_core/onnxruntime/lib/...` |
| `--dict` | Open JTalk 辞書ディレクトリ | `./voicevox_core/dict/...` |

### 実行例

キャラクターとスタイルを指定して実行する場合：

```bash
cargo run --example cli -- \
  --tts "こんにちは、なのだ。" \
  --character "ずんだもん" \
  --style "あまあま" \
  --output zundamon.wav
```
