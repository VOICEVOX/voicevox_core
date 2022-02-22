# VOICEVOX CORE

[VOICEVOX](https://voicevox.hiroshiba.jp/) の音声合成コア。  
[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にビルド済みのコアライブラリ（.so/.dll/.dylib）があります。

（エディターは [VOICEVOX](https://github.com/VOICEVOX/voicevox/) 、
エンジンは [VOICEVOX ENGINE](https://github.com/VOICEVOX/voicevox_engine/) 、
全体構成は [こちら](https://github.com/VOICEVOX/voicevox/blob/main/docs/%E5%85%A8%E4%BD%93%E6%A7%8B%E6%88%90.md) に詳細があります。）

## 環境構築
configure.pyを用いて環境構築を行う場合

```bash
python configure.py
pip install -r requirements.txt
pip install .
```
<details>
<summary>configure.pyを使わない場合</summary>

### ONNX Runtimeのダウンロード

コアを利用するにはまず環境に対応した [ONNXRUNTIME](https://github.com/microsoft/onnxruntime) をダウンロードし、リポジトリに`onnxruntime`というディレクトリ名で展開します。

動作確認済みバージョン
- ONNX Runtime v1.9.0/v1.9.1

#### GPUを使用する場合
##### CUDA
Windows, Linux上でnvidia製GPUを使用してCUDAを用いた合成を行う場合、[CUDA 11.1](https://developer.nvidia.com/cuda-11.1.0-download-archive),[CUDNN](https://developer.nvidia.com/cudnn)のインストールに加えてGPU に対応した [ONNXRUNTIME](https://github.com/microsoft/onnxruntime) のダウンロードが必要です。

##### DirectML
Windows上でDirectX12に対応したGPUを使用してDirectMLを用いた合成を行う場合、[DirectML](https://www.nuget.org/packages/Microsoft.AI.DirectML)及びDirectMLに対応した[ONNXRUNTIME](https://github.com/microsoft/onnxruntime) のダウンロードが必要です。

DirectMLは.nupkgファイルで提供されますが、拡張子を.zipに変更した上で、リポジトリに`directml`というディレクトリ名で展開してください。


#### Raspberry Pi (armhf)の場合

Raspberry Pi 用の ONNX Runtime は以下からダウンロードできます。

- <https://github.com/VOICEVOX/onnxruntime-builder/releases>

動作には、libgomp のインストールが必要です。

### コアライブラリのダウンロードと配置

まず [Releases](https://github.com/VOICEVOX/voicevox_core/releases) からコアライブラリが入った zip をダウンロードしておきます。

1. まずReleasesからダウンロードしたコアライブラリのzipを、`release`というディレクトリ名で展開する。
2. `core/lib/`ディレクトリを作成する。
3. `onnxruntime/lib`にある全てのファイルと、`release/`にある`core.h`を`core/lib/`にコピーする。
4. `release/`内にある、自身の環境に対応したランタイムライブラリを`core/lib/`にコピーし、名前をWindowsなら`core.dll`に、linuxなら`libcore.so`に、Macなら`libcore.dylib`に変更する。
    - (x64版WindowsでCPU版ライブラリを使いたいなら`core_cpu_x64.dll`を`core.dll`に変更)
5. 以下のコマンドを実行する。

```bash
# インストールに必要なモジュールのインストール
pip install -r requirements.txt
# pythonモジュールのインストール
pip install .
```

</details>

### 注意
#### GPUの使用について

##### CUDA
nvidia製GPUを搭載したWindows, Linux PCではCUDAを用いた合成が可能です。
CUDAを使用する場合、[CUDA 11.1](https://developer.nvidia.com/cuda-11.1.0-download-archive) と [CUDNN](https://developer.nvidia.com/cudnn) をインストールした上で、環境構築時、上記例の代わりに
```bash
python configure.py --use_cuda
```
を実行する必要があります

##### DirectML
DirectX12に対応したGPUを搭載したWindows PCではDirectMLを用いた合成が可能です
DirectMLを使用する場合、環境構築時、上記例の代わりに
```bash 
python configure.py --use_directml
```
を実行する必要があります

MacOSの場合、CUDA の macOS サポートは現在終了しているため、VOICEVOX CORE の macOS 向けコアライブラリも CUDA, CUDNN を利用しない CPU 版のみの提供となります。

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

## サンプル実行
```bash
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


### その他の言語

サンプルコードを実装された際はぜひお知らせください。こちらに追記させて頂きます。

## API

[core.h](./core/src/core.h) をご参照ください。

## コアライブラリのビルド

[Releases](https://github.com/Hiroshiba/voicevox_core/releases) にあるビルド済みのコアライブラリを利用せず、自分で一からビルドする場合こちらを参照してください。ビルドにはONNXRUNTIMEに加えてCMake 3.16以上が必要です。
   
modelフォルダにあるonnxモデルはダミーのため、ノイズの混じった音声が出力されます

```bash
# C++モジュールのビルド
mkdir build
cd build

# cmake .. 時のオプション
# -DONNXRUNTIME_DIR=(パス) ダウンロードしたonnxruntimeが別フォルダにある時指定
# -DDIRECTML=ON             DirectMLを使用する場合指定
# -DDIRECTML_DIR=(パス)    ダウンロードしたDirectMLが別フォルダにある時指定
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
