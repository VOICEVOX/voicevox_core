# VOICEVOX CORE

[VOICEVOX](https://voicevox.hiroshiba.jp/) の音声合成コア。  
[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にビルド済みのコアライブラリ（.so/.dll/.dylib）があります。

（エディターは [VOICEVOX](https://github.com/VOICEVOX/voicevox/) 、
エンジンは [VOICEVOX ENGINE](https://github.com/VOICEVOX/voicevox_engine/) 、
全体構成は [こちら](https://github.com/VOICEVOX/voicevox/blob/main/docs/%E5%85%A8%E4%BD%93%E6%A7%8B%E6%88%90.md) に詳細があります。）

## 環境構築

downloader を用いて環境構築を行う場合

### Windows の場合

PowerShell で下記コマンドを実行してください

```powershell
Invoke-WebRequest https://github.com/VOICEVOX/voicevox_core/releases/latest/download/Download.ps1 | powershell
```

### Linux/macOS の場合

```bash
curl -sSLo https://github.com/VOICEVOX/voicevox_core/releases/latest/download/download.sh | bash -s
```

詳細な downloader の使い方については [こちら](./docs/downloads/Download.md) を参照してください

<details>
<summary> downloader を使わない場合</summary>

<!--
#### Raspberry Pi (armhf)の場合

Raspberry Pi 用の ONNX Runtime は以下からダウンロードできます。

- <https://github.com/VOICEVOX/onnxruntime-builder/releases>

動作には、libgomp のインストールが必要です。
-->

### コアライブラリのダウンロードと配置

1. まず [Releases](https://github.com/VOICEVOX/voicevox_core/releases/latest) からダウンロードしたコアライブラリの zip を、適当なディレクトリ名で展開します。CUDA 版、DirectML 版はかならずその zip ファイルをダウンロードしてください。
2. CUDA や DirectML を利用する場合は、 [追加ライブラリ](https://github.com/VOICEVOX/voicevox_additional_libraries/releases/latest) をダウンロードして、コアライブラリを展開したディレクトリに展開してください。
3. [Open JTalk から配布されている辞書ファイル](https://jaist.dl.sourceforge.net/project/open-jtalk/Dictionary/open_jtalk_dic-1.11/open_jtalk_dic_utf_8-1.11.tar.gz) から Open JTalk の辞書ファイルをダウンロードしてコアライブラリを展開したディレクトリに展開してください。

</details>

### 注意

#### GPU の使用について

##### CUDA

nvidia 製 GPU を搭載した Windows, Linux PC では CUDA を用いた合成が可能です。

CUDA 版を利用するには専用の download コマンドの実行が必要です。  
詳細は [CUDA 版をダウンロードする場合](./docs/downloads/Download.md#cuda-版をダウンロードする場合) を参照してください

##### DirectML

DirectX12 に対応した GPU を搭載した Windows PC では DirectML を用いた合成が可能です  
DirectML 版を利用するには専用の download コマンドの実行が必要です。  
詳細は [DirectML 版をダウンロードする場合](./docs/downloads/Download.md#directml-版をダウンロードする場合) を参照してください

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

現在このリポジトリでは次のサンプルが提供されています。実行方法についてはそれぞれのディレクトリ内にある README を参照してください

- [Python](./example/python)
- [C++(UNIX CMake)](./example/cpp/unix)
- [C++(Windows Visual Studio)](./example/cpp/windows)

### その他の言語

サンプルコードを実装された際はぜひお知らせください。こちらに追記させて頂きます。

## API

[Releases](https://github.com/VOICEVOX/voicevox_core/releases)にある zip ファイル内に core.h が入っているのでご確認ください

## コアライブラリのビルド

[Releases](https://github.com/VOICEVOX/voicevox_core/releases) にあるビルド済みのコアライブラリを利用せず、自分で一からビルドする場合こちらを参照してください。ビルドには [Rust](https://www.rust-lang.org/ja) ([Windows での Rust 開発環境構築手順はこちら](https://docs.microsoft.com/ja-jp/windows/dev-environment/rust/setup)) と [cmake](https://cmake.org/download/) が必要です。

model フォルダにある onnx モデルはダミーのため、ノイズの混じった音声が出力されます

```bash
cargo build --release
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
