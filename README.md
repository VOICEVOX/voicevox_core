# VOICEVOX CORE

[VOICEVOX](https://voicevox.hiroshiba.jp/) の音声合成コア。  
[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にビルド済みのコアライブラリ（.so/.dll/.dylib）があります。

（エディターは [VOICEVOX](https://github.com/VOICEVOX/voicevox/) 、
エンジンは [VOICEVOX ENGINE](https://github.com/VOICEVOX/voicevox_engine/) 、
全体構成は [こちら](https://github.com/VOICEVOX/voicevox/blob/main/docs/%E5%85%A8%E4%BD%93%E6%A7%8B%E6%88%90.md) に詳細があります。）

## ONNX Runtimeのダウンロード

コアを利用するにはまず環境に対応した [ONNXRUNTIME](https://github.com/microsoft/onnxruntime) をダウンロードし、リポジトリに`onnxruntime`というディレクトリ名で展開します。

動作確認済みバージョン
- ONNX Runtime v1.9.0/v1.9.1

### Windows と Linux の場合

GPU 対応版は[CUDA 11.1](https://developer.nvidia.com/cuda-11.1.0-download-archive) と [CUDNN](https://developer.nvidia.com/cudnn) のインストールと GPU に対応した [ONNXRUNTIME](https://github.com/microsoft/onnxruntime) のダウンロードが必要です。

### macOS の場合

CUDA の macOS サポートは現在終了しているため、VOICEVOX CORE の macOS 向けコアライブラリも CUDA, CUDNN を利用しない CPU 版のみの提供となります。

### Raspberry Pi (armhf)の場合

Raspberry Pi 用の ONNX Runtime は以下からダウンロードできます。

- <https://github.com/VOICEVOX/onnxruntime-builder/releases>

動作には、libgomp のインストールが必要です。

```shell
sudo apt install libgomp1
```

## API

[core.h](./core/src/core.h) をご参照ください。

## サンプルの実行

まず [Releases](https://github.com/VOICEVOX/voicevox_core/releases) からコアライブラリが入った zip をダウンロードしておきます。

### Python 3

#### ソースコードから実行

1. まずReleasesからダウンロードしたコアライブラリのzipを、`release`というディレクトリ名で展開する。
2. `core/lib/`ディレクトリを作成する。
3. `onnxruntime/lib`にある全てのファイルと、`release/`にある`core.h`を`core/lib/`にコピーする。
4. `release/`内にある、自身の環境に対応したランタイムライブラリを`core/lib/`にコピーし、名前をWindowsなら`core.dll`に、linuxなら`libcore.so`に、Macなら`libcore.dylib`に変更する。
    - (x64版WindowsでCPU版ライブラリを使いたいなら`core_cpu_x64.dll`を`core.dll`に変更)
5. 以下のコマンドを実行する。

```bash
# Windowsの場合、DLLからLIBファイルの作成
example/python/makelib.bat core/lib/core

# pythonモジュールのインストール
pip install -r requirements.txt
pip install .

cd example/python

# サンプルコード実行のための依存モジュールのインストール
pip install -r requirements.txt
python run.py \
    --text "これは本当に実行できているんですか" \
    --speaker_id 1 \
    --root_dir_path="../../release"

# 引数の紹介
# --text 読み上げるテキスト
# --speaker_id 話者ID
# --use_gpu GPUを使う
# --f0_speaker_id 音高の話者ID（デフォルト値はspeaker_id）
# --f0_correct 音高の補正値（デフォルト値は0。+-0.3くらいで結果が大きく変わります）
# --root_dir_path onnxファイル等必要なファイルがあるディレクトリ
```

#### Docker から

現在 onnx 対応作業中のため、main ブランチ上の Dockerfile はビルドできません  
`git checkout origin/release-0.9`を実行し、バージョン 0.9.0 の環境でお試しください  
詳しくは[#44](https://github.com/VOICEVOX/voicevox_core/issues/44)へ

<details>

```bash
# イメージのビルド
docker build -t voicevox_core example/python

# コンテナの起動(音声を保存しておくボリュームを作成)
docker run -it -v ~/voicevox:/root/voice voicevox_core bash

# テスト音声 `おはようございます-1.wav` を生成
python run.py --text おはようございます --speaker_id 1
mv *.wav ~/voice
exit

# 音声の再生
aplay ~/voice/おはようございます-1.wav
```

</details>

### その他の言語

サンプルコードを実装された際はぜひお知らせください。こちらに追記させて頂きます。

## コアライブラリのビルド

[Releases](https://github.com/Hiroshiba/voicevox_core/releases) にあるビルド済みのコアライブラリを利用せず、自分で一からビルドする場合こちらを参照してください。ビルドにはONNXRUNTIMEに加えてCMake 3.16以上が必要です。

```bash
# C++モジュールのビルド
mkdir build
cd build
# もしダウンロードしたonnx runtimeが別のところにあるなら、以下のコマンドを
# cmake .. -DONNXRUNTIME_DIR=(ダウンロードしたonnx runtimeのパス) に変更する。
cmake ..
cmake --build . --config Release
cmake --install .
cd ..

# (省略可能) pythonモジュールのテスト
python setup.py test

# pythonモジュールのインストール
pip install .

cd example/python

python run.py \
    --text "これは本当に実行できているんですか" \
    --speaker_id 1 \
    --root_dir_path="../../model"
```

## 事例紹介

**[VOICEVOX ENGINE SHARP](https://github.com/yamachu/VoicevoxEngineSharp) [@yamachu](https://github.com/yamachu)** ･･･ VOICEVOX ENGINE の C# 実装  
**[Node VOICEVOX Engine](https://github.com/y-chan/node-voicevox-engine) [@y-chan](https://github.com/y-chan)** ･･･ VOICEVOX ENGINE の Node.js/C++ 実装

## ライセンス

ソースコードのライセンスは [MIT LICENSE](./LICENSE) です。

[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にあるビルド済みのコアライブラリは別ライセンスなのでご注意ください。
