# VOICEVOX CORE

[VOICEVOX](https://voicevox.hiroshiba.jp/) の音声合成コア。  
[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にビルド済みのコアライブラリ（.so/.dll/.dylib）があります。

（エディターは [VOICEVOX](https://github.com/VOICEVOX/voicevox/) 、
エンジンは [VOICEVOX ENGINE](https://github.com/VOICEVOX/voicevox_engine/) 、
全体構成は [こちら](https://github.com/VOICEVOX/voicevox/blob/main/docs/%E5%85%A8%E4%BD%93%E6%A7%8B%E6%88%90.md) に詳細があります。）

## 環境構築

configure.py を用いて環境構築を行う場合

```bash
python configure.py
pip install -r requirements.txt
pip install .
```

<details>
<summary>configure.pyを使わない場合</summary>

<!--
### ONNX Runtimeのダウンロード

コアを利用するにはまず環境に対応した [ONNXRUNTIME](https://github.com/microsoft/onnxruntime) をダウンロードし、リポジトリに`onnxruntime`というディレクトリ名で展開します。

動作確認済みバージョン
- ONNX Runtime v1.11.1
-->

#### GPU を使用する場合

##### CUDA

Windows, Linux 上で nvidia 製 GPU を使用して CUDA を用いた合成を行う場合、[CUDA 11.1](https://developer.nvidia.com/cuda-11.1.0-download-archive),[CUDNN](https://developer.nvidia.com/cudnn)のインストールに加えて GPU に対応した [ONNXRUNTIME](https://github.com/microsoft/onnxruntime) のダウンロードが必要です。

<!--
##### DirectML
Windows上でDirectX12に対応したGPUを使用してDirectMLを用いた合成を行う場合、[DirectML](https://www.nuget.org/packages/Microsoft.AI.DirectML)及びDirectMLに対応した[ONNXRUNTIME](https://github.com/microsoft/onnxruntime) のダウンロードが必要です。

DirectMLは.nupkgファイルで提供されますが、拡張子を.zipに変更した上で、リポジトリに`directml`というディレクトリ名で展開してください。
-->

<!--
#### Raspberry Pi (armhf)の場合

Raspberry Pi 用の ONNX Runtime は以下からダウンロードできます。

- <https://github.com/VOICEVOX/onnxruntime-builder/releases>

動作には、libgomp のインストールが必要です。
-->

### コアライブラリのダウンロードと配置

まず [Releases](https://github.com/VOICEVOX/voicevox_core/releases) からコアライブラリが入った zip をダウンロードしておきます。

1. まず Releases からダウンロードしたコアライブラリの zip を、`release`というディレクトリ名で展開する。
2. `core/lib/`ディレクトリを作成する。
3. `release/`内にある、自身の環境に対応したランタイムライブラリを`core/lib/`にコピーする

```bash
# インストールに必要なモジュールのインストール
pip install -r requirements.txt
# pythonモジュールのインストール
pip install .
```

</details>

### 注意

#### GPU の使用について

##### CUDA

nvidia 製 GPU を搭載した Windows, Linux PC では CUDA を用いた合成が可能です。
CUDA を使用する場合、[CUDA 11.1](https://developer.nvidia.com/cuda-11.1.0-download-archive) と [CUDNN](https://developer.nvidia.com/cudnn) をインストールした上で、環境構築時、上記例の代わりに

```bash
python configure.py --use_cuda
```

を実行する必要があります

<!--
##### DirectML
DirectX12に対応したGPUを搭載したWindows PCではDirectMLを用いた合成が可能です
DirectMLを使用する場合、環境構築時、上記例の代わりに

```bash
python configure.py --use_directml
```

を実行する必要があります
-->

MacOS の場合、CUDA の macOS サポートは現在終了しているため、VOICEVOX CORE の macOS 向けコアライブラリも CUDA, CUDNN を利用しない CPU 版のみの提供となります。

<!--
#### Raspberry Piでの使用について

Raspberry PiなどのarmhアーキテクチャPCでの使用では、環境構築時に https://github.com/VOICEVOX/onnxruntime-builder/releases にある独自ビルドのonnxruntimeを使用する必要があります。
そのため、環境にあったファイルのURLを取得し、上記例の代わりに
```bash
python configure.py --ort_download_link <独自ビルドonnxruntimeのURL>
```
を実行してください

また、動作には、libgomp のインストールが必要です。

```shell
sudo apt install libgomp1
```
-->

## サンプル実行

```bash
cd example/python

# サンプルコード実行のための依存モジュールのインストール
pip install -r requirements.txt
python run.py \
    --text "これは本当に実行できているんですか" \
    --speaker_id 1

# 引数の紹介
# --text 読み上げるテキスト
# --speaker_id 話者ID
# --use_gpu GPUを使う
# --f0_speaker_id 音高の話者ID（デフォルト値はspeaker_id）
# --f0_correct 音高の補正値（デフォルト値は0。+-0.3くらいで結果が大きく変わります）
```

### その他の言語

サンプルコードを実装された際はぜひお知らせください。こちらに追記させて頂きます。

## API

[Releases](https://github.com/VOICEVOX/voicevox_core/releases)にある zip ファイル内に core.h が入っているのでご確認ください

## コアライブラリのビルド

[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にあるビルド済みのコアライブラリを利用せず、自分で一からビルドする場合こちらを参照してください。ビルドには [Rust](https://www.rust-lang.org/ja) が必要です。

model フォルダにある onnx モデルはダミーのため、ノイズの混じった音声が出力されます

```bash
cargo build --release

# (省略可能) pythonモジュールのテスト
python setup.py test

# pythonモジュールのインストール
pip install .

cd example/python

python run.py \
    --text "これは本当に実行できているんですか" \
    --speaker_id 1
```

## コアライブラリのテスト

```bash
cargo test
```

## 事例紹介

**[VOICEVOX ENGINE SHARP](https://github.com/yamachu/VoicevoxEngineSharp) [@yamachu](https://github.com/yamachu)** ･･･ VOICEVOX ENGINE の C# 実装  
**[Node VOICEVOX Engine](https://github.com/y-chan/node-voicevox-engine) [@y-chan](https://github.com/y-chan)** ･･･ VOICEVOX ENGINE の Node.js/C++ 実装

## ライセンス

ソースコードのライセンスは [MIT LICENSE](./LICENSE) です。

[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にあるビルド済みのコアライブラリは別ライセンスなのでご注意ください。
